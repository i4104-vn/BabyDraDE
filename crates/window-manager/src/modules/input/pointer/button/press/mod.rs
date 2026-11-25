//! Pointer press handling.

use crate::modules::input::InputContext;

use super::models::ButtonContext;

mod layer;
mod shared;
mod ui;
mod window;

/// Handles a pointer press: tracks the button, then routes the click through
/// switcher, launcher, and window/layer interactions in priority order.
///
/// # Arguments
/// * `ctx` - Mutable button context holding state, serial, and button code.
pub(super) fn press<C: InputContext>(ctx: &mut ButtonContext<'_, C>) {
    ctx.state
        .pointer_pressed_buttons_mut()
        .insert(ctx.button_code);

    if ui::switcher(ctx) {
        return;
    }
    if ui::launcher(ctx) {
        return;
    }
    if !ctx.state.pointer_is_grabbed() {
        window::pointer(ctx);
    }
}
