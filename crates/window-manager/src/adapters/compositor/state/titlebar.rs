//! Adapts a Smithay window to the pure titlebar hit-testing rule.

use common::Action;
use smithay::{
    desktop::Window,
    reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State,
    utils::{Logical, Point},
};

use crate::domain::titlebar::{self, FrameInput};

/// Builds frame facts from a window's current protocol state.
///
/// # Arguments
///
/// * `window` - Window to inspect.
/// * `location` - Window location in logical pixels.
///
/// # Returns
///
/// `None` when the window has no toplevel surface.
pub(crate) fn frame_input(window: &Window, location: Point<i32, Logical>) -> Option<FrameInput> {
    let state = window.toplevel()?.current_state();
    Some(FrameInput {
        location,
        width: window.geometry().size.w as f64,
        maximized: state.states.contains(State::Maximized),
        server_side_decoration: state.decoration_mode == Some(Mode::ServerSide),
        fullscreen: state.states.contains(State::Fullscreen),
    })
}

/// Hit-tests a point against a window's compositor-owned frame.
///
/// # Arguments
///
/// * `window` - Window to test.
/// * `location` - Window location in logical pixels.
/// * `point` - Pointer position in global logical coordinates.
///
/// # Returns
///
/// The action under the pointer, or `None` outside a hit-testable frame.
pub(crate) fn hit_test(
    window: &Window,
    location: Point<i32, Logical>,
    point: Point<f64, Logical>,
) -> Option<Action> {
    titlebar::hit_test(frame_input(window, location)?, point)
}
