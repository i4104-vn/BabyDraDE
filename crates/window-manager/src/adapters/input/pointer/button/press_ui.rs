//! Switcher and launcher press routing.

use crate::adapters::input::InputContext;

use crate::adapters::input::pointer::button::models::ButtonContext;

/// Handles a click on the task switcher overlay.
///
/// # Arguments
/// * `ctx` - Mutable button context.
///
/// # Returns
/// `true` when the click was consumed by the switcher.
pub(super) fn switcher<C: InputContext>(ctx: &mut ButtonContext<'_, C>) -> bool {
    if ctx.state.switcher_is_active() {
        let px = ctx.state.pointer_location().x;
        let py = ctx.state.pointer_location().y;

        if let Some(clicked_idx) = ctx.state.switcher_index_at((px, py), 0.0) {
            if let Some(window) = ctx.state.switcher_window_at(clicked_idx) {
                ctx.state.unminimize_window(&window);
                crate::adapters::input::keyboard::focus::focus_window(
                    ctx.state, window, ctx.serial,
                );
            }
            ctx.state.switcher_begin_close();
            return true;
        }
    }
    false
}

/// Handles launcher search and icon clicks.
///
/// # Arguments
/// * `ctx` - Mutable button context.
///
/// # Returns
/// `true` when the click was consumed by the launcher.
pub(super) fn launcher<C: InputContext>(ctx: &mut ButtonContext<'_, C>) -> bool {
    // While the search overlay is expanded it owns the whole screen:
    // clicking a result activates it, clicking outside returns to the
    // docked launcher.
    if ctx.button_code == common::BTN_LEFT
        && !ctx.state.pointer_is_grabbed()
        && ctx
            .state
            .launcher_search_click(ctx.state.pointer_location())
    {
        return true;
    }

    // The launcher is compositor-owned UI: a click on one of its icons
    // launches that app and must not reach (or raise) any window.
    if !ctx.state.switcher_is_active()
        && ctx.button_code == common::BTN_LEFT
        && !ctx.state.pointer_is_grabbed()
    {
        if let Some(index) = ctx.state.launcher_index_at(ctx.state.pointer_location()) {
            ctx.state.launcher_launch(index);
            return true;
        }
        // Clicking elsewhere dismisses the launcher.
        if ctx.state.launcher_is_visible() {
            ctx.state.launcher_toggle();
        }
    }
    false
}
