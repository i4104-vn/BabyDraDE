//! settings helpers.

use super::*;

impl BabydraState {
    /// Reloads input device configuration from `~/.babydra/configs/input.toml`.
    ///
    /// Flushes the shared configuration cache and refreshes cached touchpad
    /// and mouse settings including natural scroll toggles.
    pub(crate) fn reload_input_config(&mut self) {
        self.config.reload();
    }
    /// Hot reloads blur and compositor configurations from disk.
    pub(crate) fn reload_config(&mut self) {
        self.render.reload_config();
    }
    /// Cursor shape the renderer should show this frame.
    ///
    /// Compositor-owned drag operations and hover targets override the client's
    /// last request: borders show the matching directional resize cursor (both
    /// on hover and while dragging), the draggable titlebar shows a grab cursor,
    /// and active moves show a grabbing cursor.
    pub(crate) fn effective_cursor_icon(&self) -> CursorIcon {
        if let Some(ref resize) = self.input.resize_state {
            return resize_cursor_icon(resize.edges);
        }
        if self.input.move_state.is_some() {
            return CursorIcon::Grabbing;
        }

        let point = self.pointer_location;

        // Hovering a resize border shows the same arrow that a drag would use.
        if let Some((_, _, edge)) = self.window_border_under(point) {
            return resize_cursor_icon(edge);
        }

        // Hovering the draggable part of a server-side titlebar shows a grab
        // cursor. Buttons keep whatever the client requested.
        if let Some((window, location)) = self.window_under(point) {
            if matches!(
                crate::adapters::compositor::state::titlebar::hit_test(&window, location, point),
                Some(common::Action::Move)
            ) {
                return CursorIcon::Grab;
            }
        }

        self.input.cursor_icon
    }
}
