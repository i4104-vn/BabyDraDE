//! sizing_fullscreen helpers.

use super::*;

impl BabydraState {
    pub(crate) fn fullscreen_window(&mut self, window: &Window, target_output: Option<WlOutput>) {
        let Some(toplevel) = window.toplevel() else {
            return;
        };

        self.unminimize_window(window);

        let current_loc = self
            .space
            .element_location(window)
            .unwrap_or_else(default_smithay_position);
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
            .unwrap_or_else(default_smithay_position);
        let current_size = window.geometry().size;

        let (saved_loc, saved_size) = {
            let mut state_guard = lock_window_state(window);
            (
                state_guard.saved_location.take(),
                state_guard.saved_size.take(),
            )
        };

        let (restore_loc, restore_size) =
            smithay_restore_target(saved_loc.zip(saved_size), current_size);
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
