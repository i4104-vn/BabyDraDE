//! Pointer release handling.

use smithay::input::pointer::MotionEvent;

use crate::modules::input::InputContext;

use super::models::ButtonContext;

pub(super) fn release<C: InputContext>(ctx: &mut ButtonContext<'_, C>) {
    ctx.state
        .pointer_pressed_buttons_mut()
        .remove(&ctx.button_code);

    if let Some(resize_state) = ctx.state.resize_state_mut().take() {
        if let Some(toplevel) = resize_state.window.toplevel() {
            toplevel.with_pending_state(|state| {
                        state.states.unset(
                            smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Resizing,
                        );
                    });
            toplevel.send_configure();
        }
    }

    if let Some(move_state) = ctx.state.move_state_mut().take() {
        if move_state.has_dragged {
            if let Some(toplevel) = move_state.window.toplevel() {
                toplevel.send_configure();
            }
        }
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
}
