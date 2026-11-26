pub(crate) use crate::modules::window::animation::{
    cancel_minimizing_animation, is_window_exiting, start_window_minimizing, start_window_opening,
    start_window_restoring, start_window_transition, window_animation_frame,
};

#[allow(clippy::module_inception)]
pub(crate) mod ops {
    //! window_ops helpers.

    use crate::protocol::{ActiveWindowInfo, Event};
    use smithay::{
        desktop::Window, reexports::wayland_server::Resource, utils::IsAlive,
        wayland::shell::xdg::ToplevelSurface,
    };

    use crate::{app::state::BabydraState, modules::window::windows::WindowAction};

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
                crate::modules::window::metadata::remove(&window);
            }
            self.windows.retain(|w| w.alive());

            if self.windows.len() != before || self.windows.is_empty() {
                let active = self.active_window();
                self.update_active_window(active.as_ref());
            }
        }
        /// Returns the topmost live window, excluding closing animations.
        pub(crate) fn active_window(&self) -> Option<Window> {
            self.windows.active_window(&self.windows.space)
        }
        /// Raises a window to the top of the internal Z-order list.
        pub(crate) fn raise_window(&mut self, window: &Window) {
            self.windows.raise(window);
        }
        /// Finds a window associated with an XDG toplevel surface.
        pub(crate) fn find_window(&self, surface: &ToplevelSurface) -> Option<Window> {
            self.windows.find_window(surface, &self.windows.space)
        }
        /// Builds the focused-window snapshot without publishing it.
        ///
        /// # Arguments
        /// * `window` - The currently focused window, if any.
        ///
        /// # Returns
        /// The window's app id, title and resolved process id, or an empty
        /// snapshot when no window or toplevel is available.
        fn active_window_info(&self, window: Option<&Window>) -> ActiveWindowInfo {
            let Some(window) = window else {
                return ActiveWindowInfo::default();
            };
            let Some(toplevel) = window.toplevel() else {
                return ActiveWindowInfo::default();
            };
            let metadata = crate::modules::window::metadata::get(window).unwrap_or_default();
            let process_id = self
                .wayland
                .display_handle
                .get_client(toplevel.wl_surface().id())
                .ok()
                .and_then(|client| client.get_credentials(&self.wayland.display_handle).ok())
                .and_then(|credentials| u32::try_from(credentials.pid).ok());
            ActiveWindowInfo {
                app_id: metadata.app_id,
                title: metadata.title,
                process_id,
            }
        }

        /// Publishes the focused-window snapshot as a retained bus event.
        ///
        /// # Arguments
        /// * `window` - The currently focused window, if any.
        pub(crate) fn update_active_window(&self, window: Option<&Window>) {
            let info = self.active_window_info(window);
            let _ = self.bus.publish_retained(
                "window.active",
                &Event::ActiveWindowChanged { info },
                std::time::Duration::from_secs(60),
            );
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
}
