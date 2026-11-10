use smithay::{
    desktop::Window,
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::protocol::wl_output::WlOutput,
    },
    utils::{IsAlive, Logical, Point, Size, SERIAL_COUNTER},
    wayland::shell::xdg::ToplevelSurface,
};

use super::BabydraState;

#[derive(Default, Debug, Clone)]
pub(crate) struct WindowStateData {
    pub(crate) saved_location: Option<Point<i32, Logical>>,
    pub(crate) saved_size: Option<Size<i32, Logical>>,
    pub(crate) is_minimized: bool,
}

pub(crate) fn window_state(window: &Window) -> &std::sync::Mutex<WindowStateData> {
    window.user_data().get_or_insert(Default::default)
}

impl BabydraState {
    pub(crate) fn cleanup_windows(&mut self) {
        self.windows.retain(|w| w.alive());
    }

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

    pub(crate) fn maximize_window(&mut self, window: &Window) {
        let Some(toplevel) = window.toplevel() else {
            return;
        };

        self.unminimize_window(window);

        let current_loc = self
            .space
            .element_location(window)
            .unwrap_or_else(|| Point::from((48, 48)));
        let current_size = window.geometry().size;

        {
            let mut state_guard = window_state(window).lock().unwrap();
            if state_guard.saved_location.is_none() {
                state_guard.saved_location = Some(current_loc);
                state_guard.saved_size = Some(current_size);
            }
        }

        let output = self
            .space
            .outputs_for_element(window)
            .into_iter()
            .next()
            .or_else(|| self.space.outputs().next().cloned());
        let Some(output) = output else {
            return;
        };
        let Some(output_geometry) = self.space.output_geometry(&output) else {
            return;
        };

        let layer_map = smithay::desktop::layer_map_for_output(&output);
        let non_exclusive = layer_map.non_exclusive_zone();

        let target_loc = output_geometry.loc + non_exclusive.loc;
        let target_size = non_exclusive.size;

        self.space.map_element(window.clone(), target_loc, true);
        self.space.raise_element(window, true);

        toplevel.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Maximized);
            state.size = Some(target_size);
        });
        toplevel.send_configure();
    }

    pub(crate) fn unmaximize_window(&mut self, window: &Window) {
        let Some(toplevel) = window.toplevel() else {
            return;
        };

        let (saved_loc, saved_size) = {
            let mut state_guard = window_state(window).lock().unwrap();
            (
                state_guard.saved_location.take(),
                state_guard.saved_size.take(),
            )
        };

        let restore_loc = saved_loc.unwrap_or_else(|| Point::from((48, 48)));
        self.space.map_element(window.clone(), restore_loc, false);

        toplevel.with_pending_state(|state| {
            state.states.unset(xdg_toplevel::State::Maximized);
            state.size = saved_size;
        });
        toplevel.send_configure();
    }

    pub(crate) fn toggle_maximize(&mut self, window: &Window) {
        let Some(toplevel) = window.toplevel() else {
            return;
        };
        let is_maximized = toplevel
            .current_state()
            .states
            .contains(xdg_toplevel::State::Maximized);
        if is_maximized {
            self.unmaximize_window(window);
        } else {
            self.maximize_window(window);
        }
    }

    pub(crate) fn minimize_window(&mut self, window: &Window) {
        {
            let mut state_guard = window_state(window).lock().unwrap();
            state_guard.is_minimized = true;
        }

        self.space.unmap_elem(window);

        self.space.elements().for_each(|w| {
            if let Some(top) = w.toplevel() {
                top.send_pending_configure();
            }
        });

        let next_top = self.space.elements().next_back().cloned();
        if let Some(ref next) = next_top {
            if let Some(top) = next.toplevel() {
                if let Some(keyboard) = self.seat.get_keyboard() {
                    keyboard.set_focus(
                        self,
                        Some(top.wl_surface().clone()),
                        SERIAL_COUNTER.next_serial(),
                    );
                }
            }
        }
        self.update_active_window(next_top.as_ref());
    }

    pub(crate) fn unminimize_window(&mut self, window: &Window) {
        let was_minimized = {
            let mut state_guard = window_state(window).lock().unwrap();
            let was = state_guard.is_minimized;
            state_guard.is_minimized = false;
            was
        };

        if was_minimized || !self.space.elements().any(|w| w == window) {
            let loc = {
                let state_guard = window_state(window).lock().unwrap();
                state_guard
                    .saved_location
                    .unwrap_or_else(|| Point::from((48, 48)))
            };
            self.space.map_element(window.clone(), loc, true);
        }

        self.space.raise_element(window, true);
        if let Some(toplevel) = window.toplevel() {
            if let Some(keyboard) = self.seat.get_keyboard() {
                keyboard.set_focus(
                    self,
                    Some(toplevel.wl_surface().clone()),
                    SERIAL_COUNTER.next_serial(),
                );
            }
            toplevel.send_pending_configure();
        }
        self.update_active_window(Some(window));
    }

    pub(crate) fn fullscreen_window(&mut self, window: &Window, target_output: Option<WlOutput>) {
        let Some(toplevel) = window.toplevel() else {
            return;
        };

        self.unminimize_window(window);

        let current_loc = self
            .space
            .element_location(window)
            .unwrap_or_else(|| Point::from((48, 48)));
        let current_size = window.geometry().size;

        {
            let mut state_guard = window_state(window).lock().unwrap();
            if state_guard.saved_location.is_none() {
                state_guard.saved_location = Some(current_loc);
                state_guard.saved_size = Some(current_size);
            }
        }

        let output = self
            .space
            .outputs_for_element(window)
            .into_iter()
            .next()
            .or_else(|| self.space.outputs().next().cloned());
        let Some(output) = output else {
            return;
        };
        let Some(output_geometry) = self.space.output_geometry(&output) else {
            return;
        };

        self.space
            .map_element(window.clone(), output_geometry.loc, true);
        self.space.raise_element(window, true);

        toplevel.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Fullscreen);
            state.size = Some(output_geometry.size);
            state.fullscreen_output = target_output;
        });
        toplevel.send_configure();
    }

    pub(crate) fn unfullscreen_window(&mut self, window: &Window) {
        let Some(toplevel) = window.toplevel() else {
            return;
        };

        let (saved_loc, saved_size) = {
            let mut state_guard = window_state(window).lock().unwrap();
            (
                state_guard.saved_location.take(),
                state_guard.saved_size.take(),
            )
        };

        let restore_loc = saved_loc.unwrap_or_else(|| Point::from((48, 48)));
        self.space.map_element(window.clone(), restore_loc, false);

        toplevel.with_pending_state(|state| {
            state.states.unset(xdg_toplevel::State::Fullscreen);
            state.fullscreen_output = None;
            state.size = saved_size;
        });
        toplevel.send_configure();
    }

    pub(crate) fn toggle_fullscreen(&mut self, window: &Window) {
        let Some(toplevel) = window.toplevel() else {
            return;
        };
        let is_fullscreen = toplevel
            .current_state()
            .states
            .contains(xdg_toplevel::State::Fullscreen);
        if is_fullscreen {
            self.unfullscreen_window(window);
        } else {
            self.fullscreen_window(window, None);
        }
    }

    pub(crate) fn close_window(&mut self, window: &Window) {
        if let Some(toplevel) = window.toplevel() {
            toplevel.send_close();
        }
    }

    pub(crate) fn handle_window_action(&mut self, action: &str) {
        let active = self.space.elements().next_back().cloned();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_state_data_lifecycle() {
        let mut state = WindowStateData::default();
        assert!(!state.is_minimized);
        assert!(state.saved_location.is_none());
        assert!(state.saved_size.is_none());

        // Maximize simulation: save floating location and size
        let initial_loc = Point::<i32, Logical>::from((100, 150));
        let initial_size = Size::<i32, Logical>::from((800, 600));
        state.saved_location = Some(initial_loc);
        state.saved_size = Some(initial_size);

        assert_eq!(state.saved_location, Some(initial_loc));
        assert_eq!(state.saved_size, Some(initial_size));

        // Restore simulation: consume saved geometry
        let restored_loc = state.saved_location.take();
        let restored_size = state.saved_size.take();
        assert_eq!(restored_loc, Some(initial_loc));
        assert_eq!(restored_size, Some(initial_size));
        assert!(state.saved_location.is_none());
        assert!(state.saved_size.is_none());

        // Minimize simulation
        state.is_minimized = true;
        assert!(state.is_minimized);
        state.is_minimized = false;
        assert!(!state.is_minimized);
    }
}
