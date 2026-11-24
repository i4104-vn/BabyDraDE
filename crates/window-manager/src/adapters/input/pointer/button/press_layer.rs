//! Layer and overlay press handling.

use smithay::wayland::shell::wlr_layer::Layer;

use crate::adapters::input::InputContext;

use crate::adapters::input::pointer::button::models::ButtonContext;

use super::shared;

/// Handles a press that did not hit an application window.
///
/// Updates pointer focus and, for non-panel layers, clears active app focus.
///
/// # Arguments
/// * `ctx` - Mutable button context.
pub(super) fn handle<C: InputContext>(ctx: &mut ButtonContext<'_, C>) {
    let clicked_layer = ctx.state.layer_under(ctx.state.pointer_location());
    let is_top_or_overlay = matches!(clicked_layer, Some(Layer::Top) | Some(Layer::Overlay));

    // Ensure pointer focus is updated to the clicked surface
    shared::refresh_pointer_focus(ctx);

    // When interacting with panels, topbar, or overlays, do NOT lose active app focus
    if !is_top_or_overlay {
        shared::dismiss_popups(ctx);

        ctx.state.space().elements().for_each(|window| {
            window.set_activated(false);
            window.toplevel().unwrap().send_pending_configure();
        });
        ctx.state.set_keyboard_focus(None, ctx.serial);
        ctx.state.update_active_window(None);
    }
}
