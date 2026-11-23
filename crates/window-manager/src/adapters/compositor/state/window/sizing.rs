use crate::application::windows::{default_position, maximize_target, restore_target};
use crate::domain::geometry::{Point as PurePoint, Rectangle as PureRectangle, Size as PureSize};
use smithay::{
    desktop::Window,
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::protocol::{wl_output::WlOutput, wl_surface::WlSurface},
    },
    utils::{Logical, Point, Rectangle, Size, SERIAL_COUNTER},
};

use super::state_data::lock_window_state;
use crate::adapters::compositor::state::{
    cancel_minimizing_animation, is_window_exiting, start_window_minimizing,
    start_window_restoring, start_window_transition, BabydraState,
};

// Smithay geometry is converted to and from pure domain values only here, at
// the boundary between the protocol adapter and the layout use cases.

fn to_pure_point(point: Point<i32, Logical>) -> PurePoint<i32> {
    PurePoint::from((point.x, point.y))
}

fn to_pure_size(size: Size<i32, Logical>) -> PureSize<i32> {
    PureSize::from((size.w, size.h))
}

fn to_pure_rect(rect: Rectangle<i32, Logical>) -> PureRectangle<i32> {
    PureRectangle::new(to_pure_point(rect.loc), to_pure_size(rect.size))
}

fn to_smithay_point(point: PurePoint<i32>) -> Point<i32, Logical> {
    Point::from((point.x, point.y))
}

fn to_smithay_size(size: PureSize<i32>) -> Size<i32, Logical> {
    Size::from((size.w, size.h))
}

fn to_smithay_rect(rect: PureRectangle<i32>) -> Rectangle<i32, Logical> {
    Rectangle::new(to_smithay_point(rect.loc), to_smithay_size(rect.size))
}

fn default_smithay_position() -> Point<i32, Logical> {
    to_smithay_point(default_position())
}

fn smithay_restore_target(
    saved: Option<(Point<i32, Logical>, Size<i32, Logical>)>,
    current_size: Size<i32, Logical>,
) -> (Point<i32, Logical>, Size<i32, Logical>) {
    let (loc, size) = restore_target(
        saved.map(|(loc, size)| (to_pure_point(loc), to_pure_size(size))),
        to_pure_size(current_size),
    );
    (to_smithay_point(loc), to_smithay_size(size))
}

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
                    if let Some(keyboard) = self.wayland.seat.get_keyboard() {
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
                if let Some(keyboard) = self.wayland.seat.get_keyboard() {
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
                state_guard.saved_location.unwrap_or_else(default_smithay_position)
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
            if let Some(keyboard) = self.wayland.seat.get_keyboard() {
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
