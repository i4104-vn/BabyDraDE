//! Shared steps of the Winit and Udev/DRM frame loops.

use std::time::Duration;

use smithay::{
    output::Output,
    reexports::wayland_server::DisplayHandle,
};

use crate::{
    adapters::compositor::BabydraState, application::window_action::WindowAction,
};

/// Runs the state updates that must happen before a frame is composed.
pub(crate) fn begin_frame(state: &mut BabydraState) {
    if let Some(raw_action) = services::system::window::read_and_clear_window_action() {
        if let Some(action) = WindowAction::parse(&raw_action) {
            state.handle_window_action(action);
        }
    }
    state.update_window_animations();
    state.update_clipboard();
}

/// Sends frame callbacks for one output.
pub(crate) fn send_frames(state: &mut BabydraState, output: &Output) {
    state.space.elements().for_each(|window| {
        window.send_frame(
            output,
            state.start_time.elapsed(),
            Some(Duration::ZERO),
            |_, _| Some(output.clone()),
        );
    });

    let layer_map = smithay::desktop::layer_map_for_output(output);
    for layer in layer_map.layers() {
        layer.send_frame(
            output,
            state.start_time.elapsed(),
            Some(Duration::ZERO),
            |_, _| Some(output.clone()),
        );
    }
}

/// Performs the shared end-of-frame bookkeeping.
pub(crate) fn end_frame(state: &mut BabydraState, display_handle: &mut DisplayHandle) {
    state.space.refresh();
    state.popups.cleanup();
    let _ = display_handle.flush_clients();
}
