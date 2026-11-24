//! window_ops helpers.

use super::*;

impl BabydraState {
    /// Removes dead windows and keeps the shared active-window state in sync.
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
    /// Returns the topmost live window, excluding closing animations.
    pub(crate) fn active_window(&self) -> Option<Window> {
        self.windows.active_window(&self.space)
    }
    /// Raises a window to the top of the internal Z-order list.
    pub(crate) fn raise_window(&mut self, window: &Window) {
        self.windows.raise(window);
    }
    /// Finds a window associated with an XDG toplevel surface.
    pub(crate) fn find_window_for_surface(&self, surface: &ToplevelSurface) -> Option<Window> {
        self.windows.find_window_for_surface(surface, &self.space)
    }
    /// Persists metadata about the currently active window.
    pub(crate) fn update_active_window(&self, window: Option<&Window>) {
        let info = if let Some(window) = window {
            if let Some(toplevel) = window.toplevel() {
                let metadata = crate::adapters::window::metadata::get(window).unwrap_or_default();
                let process_id = self
                    .wayland
                    .display_handle
                    .get_client(toplevel.wl_surface().id())
                    .ok()
                    .and_then(|client| client.get_credentials(&self.wayland.display_handle).ok())
                    .and_then(|credentials| u32::try_from(credentials.pid).ok());
                services::system::window::ActiveWindowInfo {
                    app_id: metadata.app_id,
                    title: metadata.title,
                    process_id,
                }
            } else {
                services::system::window::ActiveWindowInfo::default()
            }
        } else {
            services::system::window::ActiveWindowInfo::default()
        };
        services::system::window::save_active_window(&info);
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
