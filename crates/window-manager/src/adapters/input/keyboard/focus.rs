//! Window focus cycling and activation management for keyboard navigation.

use smithay::{desktop::Window, utils::Serial};

use super::super::InputContext;

/// Cycles focus between available windows in forward or backward order.
///
/// Unminimizes the selected window if it was minimized, raises it to the top of the space,
/// and assigns keyboard focus.
pub(crate) fn cycle_window_focus<C: InputContext>(
    state: &mut C,
    forward: bool,
    serial: Serial,
) -> bool {
    state.cleanup_windows();
    if state.windows().is_empty() {
        return false;
    }

    let windows = state.windows();
    let next_window = match (windows.len() == 1, forward) {
        (true, _) => windows.first().cloned(),
        (false, true) => windows.iter().rev().nth(1).cloned(),
        (false, false) => windows.first().cloned(),
    };

    let Some(next_window) = next_window else {
        return false;
    };

    state.unminimize_window(&next_window);
    focus_window(state, next_window, serial);
    true
}

/// Raises a window to the top of the compositor space and transfers keyboard focus to its surface.
///
/// Notifies all windows of activation state change and triggers pending configure events.
pub(crate) fn focus_window<C: InputContext>(
    state: &mut C,
    window: Window,
    serial: Serial,
) {
    let Some(toplevel) = window.toplevel() else {
        return;
    };
    if !state.has_keyboard() {
        return;
    }

    state.space_mut().raise_element(&window, true);
    state.raise_window(&window);
    state.set_keyboard_focus(Some(toplevel.wl_surface().clone()), serial);
    state.space().elements().for_each(|w| {
        let active = w == &window;
        w.set_activated(active);
        if let Some(toplevel) = w.toplevel() {
            toplevel.send_pending_configure();
        }
    });
    state.update_active_window(Some(&window));
}
