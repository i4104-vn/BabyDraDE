//! Shared steps of the Winit and Udev/DRM frame loops.
//!
//! Only compositor state and frame callbacks are handled here: protocol
//! traffic arrives through the socket-driven bus host, not frame polling.

use std::time::Duration;

use smithay::{output::Output, reexports::wayland_server::DisplayHandle};

use crate::app::state::BabydraState;

/// Runs the state updates that must happen before a frame is composed.
pub(crate) fn begin_frame(state: &mut BabydraState) {
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
