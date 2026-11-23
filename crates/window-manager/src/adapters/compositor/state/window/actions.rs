use smithay::{desktop::Window, utils::IsAlive, wayland::shell::xdg::ToplevelSurface};

use super::animation::is_window_exiting;
use crate::adapters::compositor::state::BabydraState;
use crate::application::window_action::WindowAction;

impl BabydraState {
    /// Removes dead windows from the tracking list and keeps the shared
    /// active-window state in sync (so widgets hide the name of closed apps).
    pub(crate) fn cleanup_windows(&mut self) {
        let before = self.windows.len();
        for window in self
            .windows
            .iter()
            .filter(|w| !w.alive())
            .cloned()
            .collect::<Vec<_>>()
        {
            crate::adapters::window::metadata::remove(&window);
        }
        self.windows.retain(|w| w.alive());

        if self.windows.len() != before || self.windows.is_empty() {
            let active = self.active_window();
            self.update_active_window(active.as_ref());
        }
    }

    /// Returns the topmost (most recently raised / focused) window, if any.
    ///
    /// Excludes windows that are currently playing their closing animation.
    pub(crate) fn active_window(&self) -> Option<Window> {
        self.windows
            .iter()
            .rev()
            .filter(|w| !is_window_exiting(w))
            .find(|w| w.alive())
            .cloned()
            .or_else(|| {
                self.space
                    .elements()
                    .rev()
                    .find(|w| !is_window_exiting(w))
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
    pub(crate) fn handle_window_action(&mut self, action: WindowAction) {
        let active = self.active_window();
        let Some(window) = active else {
            return;
        };
        match action {
            WindowAction::Minimize => self.minimize_window(&window),
            WindowAction::Maximize => self.toggle_maximize(&window),
            WindowAction::Fullscreen => self.toggle_fullscreen(&window),
            WindowAction::Close => self.close_window(&window),
        }
    }
}
