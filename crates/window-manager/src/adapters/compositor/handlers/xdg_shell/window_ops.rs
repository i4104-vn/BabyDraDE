use smithay::{
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel::{ResizeEdge, State},
        wayland_server::protocol::wl_output::WlOutput,
    },
    wayland::shell::xdg::ToplevelSurface,
};

use crate::adapters::compositor::state::{BabydraState, MoveState, ResizeState};

/// Starts an interactive move for the window backing `surface`.
pub(super) fn move_request(state: &mut BabydraState, surface: ToplevelSurface) {
    if state.input.pointer_pressed_buttons.is_empty() && state.input.touch_slots.is_empty() {
        tracing::debug!("Ignoring xdg_toplevel.move_request: neither pointer button nor touch contact is held down");
        return;
    }

    let Some(window) = state.find_window(&surface) else {
        return;
    };

    if let Some(toplevel) = window.toplevel() {
        if toplevel.current_state().states.contains(State::Maximized) {
            state.unmaximize_window(&window);
        }
    }

    if let Some(loc) = state.windows.space.element_location(&window) {
        state.windows.space.raise_element(&window, true);
        let drag_button = state
            .input
            .pointer_pressed_buttons
            .iter()
            .copied()
            .next()
            .unwrap_or(272);
        state.input.move_state = Some(MoveState {
            window,
            initial_window_location: loc,
            initial_pointer_location: state.input.pointer_location,
            has_dragged: false,
            button: drag_button,
        });
    }
}

/// Starts an interactive resize for the window backing `surface`.
pub(super) fn resize_request(
    state: &mut BabydraState,
    surface: ToplevelSurface,
    edges: ResizeEdge,
) {
    if state.input.pointer_pressed_buttons.is_empty() && state.input.touch_slots.is_empty() {
        return;
    }

    let Some(window) = state.find_window(&surface) else {
        return;
    };

    if let Some(toplevel) = window.toplevel() {
        if toplevel.current_state().states.contains(State::Maximized) {
            return;
        }
    }

    if let Some(loc) = state.windows.space.element_location(&window) {
        state.windows.space.raise_element(&window, true);
        let drag_button = state
            .input
            .pointer_pressed_buttons
            .iter()
            .copied()
            .next()
            .unwrap_or(272);
        let size = window.geometry().size;
        state.input.resize_state = Some(ResizeState {
            window,
            initial_window_location: loc,
            initial_window_size: size,
            initial_pointer_location: state.input.pointer_location,
            edges,
            button: drag_button,
        });
    }
}

/// Maximizes the window backing `surface`.
pub(super) fn maximize_request(state: &mut BabydraState, surface: ToplevelSurface) {
    if let Some(window) = state.find_window(&surface) {
        state.maximize_window(&window);
    }
}

/// Restores the window backing `surface` from maximized state.
pub(super) fn unmaximize_request(state: &mut BabydraState, surface: ToplevelSurface) {
    if let Some(window) = state.find_window(&surface) {
        state.unmaximize_window(&window);
    }
}

/// Minimizes the window backing `surface`.
pub(super) fn minimize_request(state: &mut BabydraState, surface: ToplevelSurface) {
    if let Some(window) = state.find_window(&surface) {
        state.minimize_window(&window);
    }
}

/// Fullscreens the window backing `surface` on the requested output.
pub(super) fn fullscreen_request(
    state: &mut BabydraState,
    surface: ToplevelSurface,
    output: Option<WlOutput>,
) {
    if let Some(window) = state.find_window(&surface) {
        state.fullscreen_window(&window, output);
    }
}

/// Restores the window backing `surface` from fullscreen state.
pub(super) fn unfullscreen_request(state: &mut BabydraState, surface: ToplevelSurface) {
    if let Some(window) = state.find_window(&surface) {
        state.unfullscreen_window(&window);
    }
}
