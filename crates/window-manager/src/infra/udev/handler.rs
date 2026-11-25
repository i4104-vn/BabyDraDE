//! Udev backend event handling: session switches, DRM events and hotplug.

use std::{cell::RefCell, error::Error, rc::Rc};

use smithay::{
    backend::{
        drm::DrmEvent,
        session::libseat::LibSeatSessionNotifier,
        udev::{UdevBackend, UdevEvent},
    },
    reexports::calloop::EventLoop,
};
use tracing::{error, info, warn};

use crate::{app::state::CalloopData, infra::models::UdevState};

/// Registers the LibSeat session notifier (pause/resume on VT switches).
pub(crate) fn register_session_notifier(
    event_loop: &mut EventLoop<'_, CalloopData>,
    state: Rc<RefCell<UdevState>>,
    notifier: LibSeatSessionNotifier,
) -> Result<(), Box<dyn Error>> {
    event_loop
        .handle()
        .insert_source(notifier, move |event, _, _data| {
            let mut s = state.borrow_mut();
            match event {
                smithay::backend::session::Event::PauseSession => {
                    info!("Session paused (switched away to another VT). Suspending scanout.");
                    s.active = false;
                    s._drm_device.pause();
                }
                smithay::backend::session::Event::ActivateSession => {
                    info!("Session activated (switched back to BabyDraDE VT). Resuming scanout.");
                    s.active = true;
                    if let Err(err) = s._drm_device.activate(false) {
                        warn!("Failed to reactivate DRM device: {err}");
                    }
                }
            }
        })
        .map_err(|err| {
            std::io::Error::other(format!("Failed to insert session notifier: {err}"))
        })?;
    Ok(())
}

/// Registers the DRM notifier (page-flip / vblank completion).
pub(crate) fn register_drm_notifier(
    event_loop: &mut EventLoop<'_, CalloopData>,
    state: Rc<RefCell<UdevState>>,
    drm_notifier: smithay::backend::drm::DrmDeviceNotifier,
) -> Result<(), Box<dyn Error>> {
    event_loop
        .handle()
        .insert_source(drm_notifier, move |event, _, _data| {
            let mut s = state.borrow_mut();
            match event {
                DrmEvent::VBlank(_crtc) => {
                    for surface_data in s.surfaces.iter_mut() {
                        let _ = surface_data.compositor.frame_submitted();
                    }
                }
                DrmEvent::Error(err) => {
                    error!("DRM device event error: {err}");
                }
            }
        })
        .map_err(|err| std::io::Error::other(format!("Failed to insert DRM notifier: {err}")))?;
    Ok(())
}

/// Registers the Udev monitor for device hotplugging.
pub(crate) fn register_udev_monitor(
    event_loop: &mut EventLoop<'_, CalloopData>,
    udev_backend: UdevBackend,
) -> Result<(), Box<dyn Error>> {
    event_loop
        .handle()
        .insert_source(udev_backend, move |event, _, _data| match event {
            UdevEvent::Added { device_id, path } => {
                info!("GPU device added: dev_id={device_id}, path={path:?}");
            }
            UdevEvent::Changed { device_id } => {
                info!("DRM device changed (possible monitor hotplug): dev_id={device_id}");
            }
            UdevEvent::Removed { device_id } => {
                warn!("GPU device removed: dev_id={device_id}");
            }
        })
        .map_err(|err| {
            std::io::Error::other(format!("Failed to insert udev event source: {err}"))
        })?;
    Ok(())
}
