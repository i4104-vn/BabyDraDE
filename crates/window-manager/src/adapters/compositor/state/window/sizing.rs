use crate::application::windows::{default_position, maximize_target, restore_target};
use smithay::{
    desktop::Window,
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::protocol::{wl_output::WlOutput, wl_surface::WlSurface},
    },
    utils::{Rectangle, SERIAL_COUNTER},
};

use super::state_data::lock_window_state;
use crate::adapters::compositor::state::{
    cancel_minimizing_animation, is_window_exiting, start_window_minimizing,
    start_window_restoring, start_window_transition, BabydraState,
};

impl BabydraState {
    pub(crate) fn maximize_window(&mut self, window: &Window) {
        let Some(toplevel) = window.toplevel() else {
            return;
        };

        self.unminimize_window(window);

        let current_loc = self
            .space
            .element_location(window)
            .unwrap_or_else(default_position);
        let current_size = window.geometry().size;

        {
            let mut state_guard = lock_window_state(window);
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

        let target = maximize_target(output_geometry, non_exclusive);
        let target_loc = target.loc;
        let target_size = target.size;

        self.space.map_element(window.clone(), target_loc, true);
        self.space.raise_element(window, true);
        self.raise_window(window);
        self.space.refresh();

        toplevel.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Maximized);
            state.size = Some(target_size);
        });
        toplevel.send_configure();

        start_window_transition(
            window,
            Rectangle::new(current_loc, current_size),
            Rectangle::new(target_loc, target_size),
        );
    }

    pub(crate) fn unmaximize_window(&mut self, window: &Window) {
        let Some(toplevel) = window.toplevel() else {
            return;
        };

        let current_loc = self
            .space
            .element_location(window)
            .unwrap_or_else(default_position);
        let current_size = window.geometry().size;

        let (saved_loc, saved_size) = {
            let mut state_guard = lock_window_state(window);
            (
                state_guard.saved_location.take(),
                state_guard.saved_size.take(),
            )
        };

        let (restore_loc, restore_size) = restore_target(saved_loc.zip(saved_size), current_size);
        self.space.map_element(window.clone(), restore_loc, false);

        toplevel.with_pending_state(|state| {
            state.states.unset(xdg_toplevel::State::Maximized);
            state.size = Some(restore_size);
        });
        toplevel.send_configure();

        start_window_transition(
            window,
            Rectangle::new(current_loc, current_size),
            Rectangle::new(restore_loc, restore_size),
        );
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
        let current_loc = self.space.element_location(window);
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
            .space
            .elements()
            .rev()
            .find(|w| *w != window && !is_window_exiting(w))
            .cloned();
        match next_top.as_ref() {
            Some(next) => {
                next.set_activated(true);
                if let Some(top) = next.toplevel() {
                    if let Some(keyboard) = self.seat.get_keyboard() {
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
                if let Some(keyboard) = self.seat.get_keyboard() {
                    keyboard.set_focus(
                        self,
                        Option::<WlSurface>::None,
                        SERIAL_COUNTER.next_serial(),
                    );
                }
            }
        }

        self.space.elements().for_each(|w| {
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

        if was_minimized || !self.space.elements().any(|w| w == window) {
            let loc = {
                let state_guard = lock_window_state(window);
                state_guard.saved_location.unwrap_or_else(default_position)
            };
            // If the shrink animation is still running, abort it so it cannot
            // unmap the window we are bringing back.
            cancel_minimizing_animation(window);
            self.space.map_element(window.clone(), loc, true);
        }

        if was_minimized {
            start_window_restoring(window);
        }

        self.space.raise_element(window, true);
        self.raise_window(window);
        self.space.refresh();

        self.space.elements().for_each(|w| {
            let active = w == window;
            w.set_activated(active);
            if let Some(toplevel) = w.toplevel() {
                toplevel.send_pending_configure();
            }
        });

        if let Some(toplevel) = window.toplevel() {
            if let Some(keyboard) = self.seat.get_keyboard() {
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

    pub(crate) fn fullscreen_window(&mut self, window: &Window, target_output: Option<WlOutput>) {
        let Some(toplevel) = window.toplevel() else {
            return;
        };

        self.unminimize_window(window);

        let current_loc = self
            .space
            .element_location(window)
            .unwrap_or_else(default_position);
        let current_size = window.geometry().size;

        {
            let mut state_guard = lock_window_state(window);
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

        start_window_transition(
            window,
            Rectangle::new(current_loc, current_size),
            Rectangle::new(output_geometry.loc, output_geometry.size),
        );
    }

    pub(crate) fn unfullscreen_window(&mut self, window: &Window) {
        let Some(toplevel) = window.toplevel() else {
            return;
        };

        let current_loc = self
            .space
            .element_location(window)
            .unwrap_or_else(default_position);
        let current_size = window.geometry().size;

        let (saved_loc, saved_size) = {
            let mut state_guard = lock_window_state(window);
            (
                state_guard.saved_location.take(),
                state_guard.saved_size.take(),
            )
        };

        let (restore_loc, restore_size) = restore_target(saved_loc.zip(saved_size), current_size);
        self.space.map_element(window.clone(), restore_loc, false);

        toplevel.with_pending_state(|state| {
            state.states.unset(xdg_toplevel::State::Fullscreen);
            state.fullscreen_output = None;
            state.size = Some(restore_size);
        });
        toplevel.send_configure();

        start_window_transition(
            window,
            Rectangle::new(current_loc, current_size),
            Rectangle::new(restore_loc, restore_size),
        );
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
}
