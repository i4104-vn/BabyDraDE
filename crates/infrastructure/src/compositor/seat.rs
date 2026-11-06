use domain::{ButtonState, KeyCombo, KeyState, Point};

use super::state::BabyDraCompositorState;

/// Handles seat input routing (keyboard and pointer).
pub struct SeatHandler;

impl SeatHandler {
    /// Dispatches a key press/release event.
    pub fn handle_key(
        state: &mut BabyDraCompositorState,
        combo: &KeyCombo,
        key_state: KeyState,
    ) -> bool {
        if key_state == KeyState::Pressed {
            let focused = state.window_manager.focused_window_id();
            if let Some(cmd) = state.input_dispatcher.dispatch(combo, focused) {
                tracing::info!("Keybinding triggered: {} -> {:?}", combo, cmd);
                state.handle_command(cmd);
                return true;
            }
        }
        false
    }

    /// Handles pointer motion across the screen.
    pub fn handle_pointer_motion(
        _state: &mut BabyDraCompositorState,
        _position: Point,
    ) {
        // Track pointer position or perform window move/resize if dragging
    }

    /// Handles pointer clicks (e.g. left click to focus/raise).
    pub fn handle_pointer_button(
        state: &mut BabyDraCompositorState,
        position: Point,
        _button: u32,
        button_state: ButtonState,
    ) {
        if button_state == ButtonState::Pressed {
            // Find window at position and focus it
            let clicked_window = state
                .window_manager
                .windows()
                .find(|w| w.geometry.contains_point(position))
                .map(|w| w.id);

            if let Some(id) = clicked_window {
                state.window_manager.focus_window(id);
            }
        }
    }
}
