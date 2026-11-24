//! Touch lifecycle handling: up, cancel, and frame.

use smithay::{
    backend::input::{InputBackend, TouchCancelEvent, TouchFrameEvent, TouchUpEvent},
    input::touch::UpEvent,
    utils::SERIAL_COUNTER,
};

use crate::adapters::input::{
    touch::events::{gesture_state, overlay},
    InputContext,
};

/// Handles lifting of a finger from the touchscreen surface (touch up).
///
/// # Arguments
/// * `state` - Mutable input context driving the compositor.
/// * `event` - Backend touch-up event carrying the released slot.
pub(crate) fn handle_touch_up<B: InputBackend, T: TouchUpEvent<B>>(
    state: &mut impl InputContext,
    event: T,
) {
    // Evaluate multi-touch gesture before removing the slot.
    gesture_state::track_up(state, event.slot(), event.time_msec() as u64);

    gesture_state::end_contact(state, event.slot());

    if overlay::handle_overlay_up(state, event.slot()) {
        return;
    }

    if gesture_state::no_contacts(state) {
        gesture_state::finish_interactions(state);
    }

    let serial = SERIAL_COUNTER.next_serial();
    state.touch_up(&UpEvent {
        slot: event.slot(),
        serial,
        time: event.time_msec(),
    });
    state.touch_frame();
}

/// Handles touchscreen contact cancellation (e.g. palm detection or gesture takeover).
///
/// # Arguments
/// * `state` - Mutable input context driving the compositor.
/// * `event` - Backend touch-cancel event carrying the cancelled slot.
pub(crate) fn handle_touch_cancel<B: InputBackend, T: TouchCancelEvent<B>>(
    state: &mut impl InputContext,
    event: T,
) {
    gesture_state::end_contact(state, event.slot());

    if overlay::handle_overlay_cancel(state, event.slot()) {
        return;
    }

    if gesture_state::no_contacts(state) {
        gesture_state::finish_interactions(state);
    }

    state.touch_cancel();
    state.touch_frame();
}

/// Flushes and completes a group of simultaneous multi-touch events.
///
/// # Arguments
/// * `state` - Mutable input context driving the compositor.
/// * `_event` - Backend touch-frame event (unused).
pub(crate) fn handle_touch_frame<B: InputBackend, T: TouchFrameEvent<B>>(
    state: &mut impl InputContext,
    _event: T,
) {
    state.touch_frame();
}
