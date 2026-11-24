//! Pointer button dispatch.

use smithay::{
    backend::input::{ButtonState, InputBackend, PointerButtonEvent},
    input::pointer::ButtonEvent,
    utils::SERIAL_COUNTER,
};

use crate::adapters::input::InputContext;

use super::models::ButtonContext;
use super::{press, release};

/// Handles pointer button clicks, focus switching, window raising, and Super+Drag initiation.
pub(crate) fn handle_pointer_button<B: InputBackend, P: PointerButtonEvent<B>>(
    state: &mut impl InputContext,
    event: P,
) {
    state.set_cursor_visible(true);
    let serial = SERIAL_COUNTER.next_serial();
    let button_code = event.button_code();
    let time = event.time_msec();
    let mut ctx = ButtonContext {
        state,
        serial,
        button_code,
        time,
    };

    match event.state() {
        ButtonState::Pressed => press::press(&mut ctx),
        ButtonState::Released => release::release(&mut ctx),
    }

    ctx.state.pointer_button(&ButtonEvent {
        button: ctx.button_code,
        state: event.state(),
        serial: ctx.serial,
        time: ctx.time,
    });
    ctx.state.pointer_frame();
}
