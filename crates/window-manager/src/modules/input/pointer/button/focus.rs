//! Pointer focus updates used by compositor-handled button events.

use smithay::{
    input::pointer::MotionEvent,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
};

use crate::modules::input::InputContext;

use super::ButtonContext;

/// Sends a motion/frame pair with the given focus so pointer focus changes
/// take effect before the button is dispatched.
fn update_focus<C: InputContext>(
    ctx: &mut ButtonContext<'_, C>,
    focus: Option<(WlSurface, Point<f64, Logical>)>,
) {
    ctx.state.pointer_motion(
        focus,
        &MotionEvent {
            location: ctx.state.pointer_location(),
            serial: ctx.serial,
            time: ctx.time,
        },
    );
    ctx.state.pointer_frame();
}

/// Refreshes pointer focus to the client surface under the pointer.
///
/// # Arguments
/// * `ctx` - Mutable button context.
pub(super) fn refresh<C: InputContext>(ctx: &mut ButtonContext<'_, C>) {
    let under = ctx.state.surface_under(ctx.state.pointer_location());
    update_focus(ctx, under);
}

/// Clears pointer focus so no client surface receives the button event.
///
/// Compositor-drawn frames (titlebars, resize borders) are not client
/// surfaces; leaving the old focus in place would deliver the press to
/// whatever window happens to sit behind them.
///
/// # Arguments
/// * `ctx` - Mutable button context.
pub(super) fn clear<C: InputContext>(ctx: &mut ButtonContext<'_, C>) {
    update_focus(ctx, None);
}
