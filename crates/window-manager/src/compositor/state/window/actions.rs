use smithay::{desktop::Window, utils::IsAlive, wayland::shell::xdg::ToplevelSurface};

use super::animation::is_window_closing;
use crate::compositor::state::BabydraState;

impl BabydraState {
    /// Removes dead windows from the tracking list.
    pub(crate) fn cleanup_windows(&mut self) {
        self.windows.retain(|w| w.alive());
    }

    /// Returns the topmost (most recently raised / focused) window, if any.
    ///
    /// Excludes windows that are currently playing their closing animation.
    pub(crate) fn active_window(&self) -> Option<Window> {
        self.windows
            .iter()
            .rev()
            .filter(|w| !is_window_closing(w))
            .find(|w| w.alive())
            .cloned()
            .or_else(|| {
                self.space
                    .elements()
                    .rev()
                    .find(|w| !is_window_closing(w))
                    .cloned()
            })
    }

    /// Raises a window to the top of the internal Z-order list.
    pub(crate) fn raise_window(&mut self, window: &Window) {
        if let Some(pos) = self.windows.iter().position(|w| w == window) {
            let win = self.windows.remove(pos);
            self.windows.push(win);
        }
    }

    /// Finds a window associated with an XDG toplevel surface.
    pub(crate) fn find_window_for_surface(&self, surface: &ToplevelSurface) -> Option<Window> {
        self.windows
            .iter()
            .find(|w| w.toplevel().map(|t| t == surface).unwrap_or(false))
            .cloned()
            .or_else(|| {
                self.space
                    .elements()
                    .find(|w| w.toplevel().map(|t| t == surface).unwrap_or(false))
                    .cloned()
            })
    }

    /// Handles system-wide window commands (e.g. from topbar, dock, gestures).
    pub(crate) fn handle_window_action(&mut self, action: &str) {
        let active = self.active_window();
        let Some(window) = active else {
            return;
        };
        match action {
            "minimize" => self.minimize_window(&window),
            "maximize" => self.toggle_maximize(&window),
            "fullscreen" => self.toggle_fullscreen(&window),
            "close" => self.close_window(&window),
            _ => {}
        }
    }
}
