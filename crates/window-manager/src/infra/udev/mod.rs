//! Udev/DRM backend: session context, hotplug handling, GPU init and rendering.

#[allow(unused_imports)]
pub(crate) mod context;
pub(crate) mod gpu;
pub(crate) mod handler;
pub(crate) mod render;

#[allow(unused_imports)]
pub(crate) use context::claim_seat;
#[allow(unused_imports)]
pub(crate) use gpu::{build_surfaces, open_gpu, scan_and_init_gpu, try_init_gpu};
#[allow(unused_imports)]
pub(crate) use handler::{
    register_drm_notifier, register_session_notifier, register_udev_monitor,
};
#[allow(unused_imports)]
pub(crate) use render::{insert_frame_timer, render_frame, setup_render_loop};

use std::{cell::RefCell, error::Error, rc::Rc};

use services::theme::DesktopTheme;
use smithay::reexports::calloop::EventLoop;
use tracing::info;

use crate::{app::state::CalloopData, modules::window::cursor_impl::Cursor};

// Compatibility aliases for the old `udev::session` / `udev::surface` paths.
pub(crate) use crate::infra::models::{UdevSession, UdevState};

pub(crate) fn init(
    event_loop: &mut EventLoop<'_, CalloopData>,
    data: &mut CalloopData,
) -> Result<(), Box<dyn Error>> {
    info!("Initializing BabyDraDE Udev/DRM backend for bare TTY/hardware scanout...");

    // 1. Establish LibSeat session & Libinput input backend
    let mut udev_session = UdevSession::new(event_loop)?;

    // 2. Scan DRM GPU and setup displays & renderers
    let gpu_result = scan_and_init_gpu(&mut udev_session.session, &udev_session.seat_name, data)?;

    let theme = DesktopTheme::default();

    let state = Rc::new(RefCell::new(UdevState {
        surfaces: gpu_result.surfaces,
        renderer: gpu_result.renderer,
        theme,
        cursor: Cursor::new(),
        active: true,
        _drm_device: gpu_result.drm_device,
        _gbm_device: gpu_result.gbm_device,
        _session: udev_session.session,
    }));

    // 3. Setup Session Notifier, DRM Event Notifier, Udev Event Monitor, and Frame Render Loop
    setup_render_loop(
        event_loop,
        state,
        udev_session.notifier,
        gpu_result.drm_notifier,
        gpu_result.udev_backend,
    )?;

    info!("BabyDraDE Udev/DRM backend initialized successfully. Hardware scanout active.");
    Ok(())
}
