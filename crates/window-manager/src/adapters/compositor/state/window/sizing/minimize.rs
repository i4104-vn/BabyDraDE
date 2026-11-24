//! sizing_minimize helpers.

use super::*;

impl BabydraState {
    pub(crate) fn minimize_window(&mut self, window: &Window) {
        let current_loc = self.windows.space.element_location(window);
        {
            let mut state_guard = lock_window_state(window);
            state_guard.is_minimized = true;
            if let Some(loc) = current_loc {
                state_guard.saved_location = Some(loc);
            }
        }

        // Keep the window mapped while it collapses; `update_window_animations`
        // unmaps it once the shrink finishes.
        start_window_minimizing(window);

        let next_top = self
            .windows
            .space
            .elements()
            .rev()
            .find(|w| *w != window && !is_window_exiting(w))
            .cloned();
        match next_top.as_ref() {
            Some(next) => {
                next.set_activated(true);
                if let Some(top) = next.toplevel() {
                    if let Some(keyboard) = self.wayland.seat().get_keyboard() {
                        keyboard.set_focus(
                            self,
                            Some(top.wl_surface().clone()),
                            SERIAL_COUNTER.next_serial(),
                        );
                    }
                    top.send_configure();
                }
            }
            None => {
                if let Some(keyboard) = self.wayland.seat().get_keyboard() {
                    keyboard.set_focus(
                        self,
                        Option::<WlSurface>::None,
                        SERIAL_COUNTER.next_serial(),
                    );
                }
            }
        }

        self.windows.space.elements().for_each(|w| {
            if next_top.as_ref() != Some(w) {
                w.set_activated(false);
            }
            if let Some(top) = w.toplevel() {
                top.send_pending_configure();
            }
        });
        self.update_active_window(next_top.as_ref());
    }
    pub(crate) fn unminimize_window(&mut self, window: &Window) {
        let was_minimized = {
            let mut state_guard = lock_window_state(window);
            let was = state_guard.is_minimized;
            state_guard.is_minimized = false;
            was
        };

        if was_minimized || !self.windows.space.elements().any(|w| w == window) {
            let loc = {
                let state_guard = lock_window_state(window);
                state_guard
                    .saved_location
                    .unwrap_or_else(default_smithay_position)
            };
            // If the shrink animation is still running, abort it so it cannot
            // unmap the window we are bringing back.
            cancel_minimizing_animation(window);
            self.windows.space.map_element(window.clone(), loc, true);
        }

        if was_minimized {
            start_window_restoring(window);
        }

        self.windows.space.raise_element(window, true);
        self.raise_window(window);
        self.windows.space.refresh();

        self.windows.space.elements().for_each(|w| {
            let active = w == window;
            w.set_activated(active);
            if let Some(toplevel) = w.toplevel() {
                toplevel.send_pending_configure();
            }
        });

        if let Some(toplevel) = window.toplevel() {
            if let Some(keyboard) = self.wayland.seat().get_keyboard() {
                keyboard.set_focus(
                    self,
                    Some(toplevel.wl_surface().clone()),
                    SERIAL_COUNTER.next_serial(),
                );
            }
            toplevel.send_configure();
        }
        self.update_active_window(Some(window));
    }
}
