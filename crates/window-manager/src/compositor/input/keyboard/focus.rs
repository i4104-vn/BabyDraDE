//! Window focus cycling and activation management for keyboard navigation.

use smithay::utils::Serial;

use crate::compositor::state::BabydraState;

/// Cycles focus between available windows in forward or backward order.
///
/// Unminimizes the selected window if it was minimized, raises it to the top of the space,
/// and assigns keyboard focus.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `forward` - `true` to cycle to the next window, `false` for the previous window.
/// * `serial` - Unique input serial counter value for the focus transition.
///
/// # Returns
///
/// Returns `true` if a window was successfully focused, or `false` if no windows exist.
pub(crate) fn cycle_window_focus(state: &mut BabydraState, forward: bool, serial: Serial) -> bool {
    state.cleanup_windows();
    if state.windows.is_empty() {
        return false;
    }

    let next_window = if state.windows.len() == 1 {
        state.windows.first().cloned()
    } else if forward {
        state.windows.iter().rev().nth(1).cloned()
    } else {
        state.windows.first().cloned()
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
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `window` - The desktop [`Window`](smithay::desktop::Window) to raise and focus.
/// * `serial` - Serial identifier associated with the keyboard focus event.
pub(crate) fn focus_window(
    state: &mut BabydraState,
    window: smithay::desktop::Window,
    serial: Serial,
) {
    let Some(toplevel) = window.toplevel() else {
        return;
    };
    let Some(keyboard) = state.seat.get_keyboard() else {
        return;
    };

    state.space.raise_element(&window, true);
    state.raise_window(&window);
    keyboard.set_focus(state, Some(toplevel.wl_surface().clone()), serial);
    state.space.elements().for_each(|w| {
        let active = w == &window;
        w.set_activated(active);
        if let Some(toplevel) = w.toplevel() {
            toplevel.send_pending_configure();
        }
    });
    state.update_active_window(Some(&window));
}
