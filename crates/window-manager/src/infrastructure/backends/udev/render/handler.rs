//! Registration of the udev backend event sources.

use std::{cell::RefCell, error::Error, rc::Rc};

use smithay::{
    backend::{drm::DrmEvent, udev::UdevEvent},
    reexports::calloop::EventLoop,
};
use tracing::{error, info, warn};

use super::super::surface::UdevState;
use crate::adapters::compositor::CalloopData;

/// Registers session, DRM, udev and frame event sources on the event loop.
///
/// # Arguments
/// * `event_loop` - The compositor event loop to attach sources to.
/// * `state` - Shared udev state updated by the session and DRM notifiers.
/// * `notifier` - LibSeat session notifier for VT switches.
/// * `drm_notifier` - DRM notifier for page-flip and vblank events.
/// * `udev_backend` - Udev monitor for device hotplugging.
///
/// # Returns
/// `Ok(())` when all sources were inserted, otherwise an error describing the failure.
pub(crate) fn setup_render_loop(
    event_loop: &mut EventLoop<'_, CalloopData>,
    state: Rc<RefCell<UdevState>>,
    notifier: smithay::backend::session::libseat::LibSeatSessionNotifier,
    drm_notifier: smithay::backend::drm::DrmDeviceNotifier,
    udev_backend: smithay::backend::udev::UdevBackend,
) -> Result<(), Box<dyn Error>> {
    // 1. Session Notifier (pause/resume when switching VTs with Ctrl+Alt+Fn)
    let state_session = state.clone();
    event_loop
        .handle()
        .insert_source(notifier, move |event, _, _data| {
            let mut s = state_session.borrow_mut();
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

    // 2. DRM event notifier (page-flip / vblank completion)
    let state_drm = state.clone();
    event_loop
        .handle()
        .insert_source(drm_notifier, move |event, _, _data| {
            let mut s = state_drm.borrow_mut();
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

    // 3. Udev event monitor for device hotplugging
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

    // 4. Frame render loop — paced for 120 FPS via the shared frame budget.
    super::frame::insert_frame_timer(event_loop, state)?;

    Ok(())
}
