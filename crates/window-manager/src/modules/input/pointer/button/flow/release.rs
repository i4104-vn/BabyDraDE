//! Pointer release handling.

use crate::modules::input::InputContext;

use super::{focus, ButtonContext};

pub(super) fn release<C: InputContext>(ctx: &mut ButtonContext<'_, C>) {
    ctx.state
        .pointer_pressed_buttons_mut()
        .remove(&ctx.button_code);

    let mut compositor_interaction = false;

    if let Some(resize_state) = ctx.state.resize_state_mut().take() {
        if let Some(toplevel) = resize_state.window.toplevel() {
            toplevel.with_pending_state(|state| {
                        state.states.unset(
                            smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Resizing,
                        );
                    });
            toplevel.send_configure();
        }
        compositor_interaction = true;
    }

    if let Some(move_state) = ctx.state.move_state_mut().take() {
        if move_state.has_dragged {
            if let Some(toplevel) = move_state.window.toplevel() {
                toplevel.send_configure();
            }
        }
        compositor_interaction = true;
    }

    if compositor_interaction {
        // The compositor ran an interactive move/resize. The release must not
        // be delivered to the surface now under the pointer (which is often
        // the window behind the one that was dragged). Clear pointer focus so
        // the client receives a proper leave, then suppress the button.
        focus::clear(ctx);
        ctx.consume();
    }
}
