//! Shared helpers for pointer press handling.

use smithay::desktop::{layer_map_for_output, PopupManager, Window};

use crate::adapters::input::InputContext;

use crate::adapters::input::pointer::button::models::ButtonContext;

/// Raises a window, focuses its toplevel, and activates its element.
///
/// # Arguments
/// * `ctx` - Mutable button context.
/// * `window` - Window to raise and focus.
pub(super) fn raise_and_focus<C: InputContext>(ctx: &mut ButtonContext<'_, C>, window: &Window) {
    ctx.state.space_mut().raise_element(window, true);
    ctx.state.raise_window(window);
    ctx.state.set_keyboard_focus(
        Some(window.toplevel().unwrap().wl_surface().clone()),
        ctx.serial,
    );
    ctx.state.space().elements().for_each(|w| {
        let active = w == window;
        w.set_activated(active);
        if let Some(top) = w.toplevel() {
            top.send_pending_configure();
        }
    });
}

/// Dismisses popups belonging to layer surfaces on every output.
///
/// # Arguments
/// * `ctx` - Mutable button context.
pub(super) fn dismiss_popups<C: InputContext>(ctx: &mut ButtonContext<'_, C>) {
    for output in ctx.state.space().outputs() {
        let layer_map = layer_map_for_output(output);
        for layer in layer_map.layers() {
            let surface = layer.wl_surface();
            for (popup, _) in PopupManager::popups_for_surface(surface) {
                let _ = PopupManager::dismiss_popup(surface, &popup);
            }
        }
    }
}

/// Sends a pointer motion/frame pair to refresh focus under the pointer.
///
/// # Arguments
/// * `ctx` - Mutable button context.
pub(super) fn refresh_pointer_focus<C: InputContext>(ctx: &mut ButtonContext<'_, C>) {
    use smithay::input::pointer::MotionEvent;

    let under = ctx.state.surface_under(ctx.state.pointer_location());
    ctx.state.pointer_motion(
        under,
        &MotionEvent {
            location: ctx.state.pointer_location(),
            serial: ctx.serial,
            time: ctx.time,
        },
    );
    ctx.state.pointer_frame();
}
