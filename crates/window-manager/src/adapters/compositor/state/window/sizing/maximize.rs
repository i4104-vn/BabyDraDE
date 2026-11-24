//! sizing_maximize helpers.

use super::*;

impl BabydraState {
    pub(crate) fn maximize_window(&mut self, window: &Window) {
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

        let layer_map = smithay::desktop::layer_map_for_output(&output);
        let non_exclusive = layer_map.non_exclusive_zone();

        let target = to_smithay_rect(maximize_target(
            to_pure_rect(output_geometry),
            to_pure_rect(non_exclusive),
        ));
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
}
