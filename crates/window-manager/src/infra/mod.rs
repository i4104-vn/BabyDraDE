//! Infrastructure adapters: backends, config loaders and process drivers.

pub(crate) mod models;
pub(crate) mod process;
pub(crate) mod udev;
pub(crate) mod winit;

use smithay::reexports::calloop::EventLoop;
use std::error::Error;
use tracing::info;

use crate::app::state::CalloopData;

pub(crate) fn init(
    event_loop: &mut EventLoop<'_, CalloopData>,
    data: &mut CalloopData,
    host_has_display: bool,
) -> Result<(), Box<dyn Error>> {
    if host_has_display {
        info!("Active graphical host display detected. Initializing nested Winit backend.");
        winit::init(event_loop, data)
    } else {
        info!("No host display detected. Initializing autonomous Udev/DRM hardware backend for direct TTY scanout.");
        udev::init(event_loop, data)
    }
}

pub(crate) mod frame_pacing {
//! Shared steps of the Winit and Udev/DRM frame loops.

use std::time::Duration;

use smithay::{output::Output, reexports::wayland_server::DisplayHandle};

use crate::app::state::BabydraState;

/// Runs the state updates that must happen before a frame is composed.
pub(crate) fn begin_frame(state: &mut BabydraState) {
    // Every window action arrives through the typed protocol bridge.
    crate::app::bridge::poll(state);
    state.update_window_animations();
    state.update_clipboard();
}

/// Sends frame callbacks for one output.
pub(crate) fn send_frames(state: &mut BabydraState, output: &Output) {
    state.windows.space.elements().for_each(|window| {
        window.send_frame(
            output,
            state.resources.start_time.elapsed(),
            Some(Duration::ZERO),
            |_, _| Some(output.clone()),
        );
    });

    let layer_map = smithay::desktop::layer_map_for_output(output);
    for layer in layer_map.layers() {
        layer.send_frame(
            output,
            state.resources.start_time.elapsed(),
            Some(Duration::ZERO),
            |_, _| Some(output.clone()),
        );
    }
}

/// Performs the shared end-of-frame bookkeeping.
pub(crate) fn end_frame(state: &mut BabydraState, display_handle: &mut DisplayHandle) {
    state.windows.space.refresh();
    state.resources.popups.cleanup();
    let _ = display_handle.flush_clients();
}
}
