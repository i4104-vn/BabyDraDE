//! Touchpad pinch and hold gesture event processing.

use smithay::{
    backend::input::{GestureBeginEvent, GestureEndEvent, InputBackend},
    input::pointer::{
        GestureHoldBeginEvent, GestureHoldEndEvent, GesturePinchBeginEvent, GesturePinchEndEvent,
        GesturePinchUpdateEvent,
    },
    utils::SERIAL_COUNTER,
};

use super::super::InputContext;

/// Handles the start of a pinch gesture on the touchpad.
pub(crate) fn handle_pinch_begin<B, E>(state: &mut impl InputContext, event: E)
where
    B: InputBackend,
    E: smithay::backend::input::GesturePinchBeginEvent<B> + GestureBeginEvent<B>,
{
    if state.has_pointer() {
        state.gesture_pinch_begin(&GesturePinchBeginEvent {
            serial: SERIAL_COUNTER.next_serial(),
            time: event.time_msec(),
            fingers: event.fingers(),
        });
    }
}

/// Handles incremental scale, rotation, and translation updates during a pinch gesture.
pub(crate) fn handle_pinch_update<B, E>(state: &mut impl InputContext, event: E)
where
    B: InputBackend,
    E: smithay::backend::input::GesturePinchUpdateEvent<B>,
{
    if state.has_pointer() {
        state.gesture_pinch_update(&GesturePinchUpdateEvent {
            time: event.time_msec(),
            delta: event.delta(),
            scale: event.scale(),
            rotation: event.rotation(),
        });
    }
}

/// Handles the completion or cancellation of a touchpad pinch gesture.
pub(crate) fn handle_pinch_end<B, E>(state: &mut impl InputContext, event: E)
where
    B: InputBackend,
    E: smithay::backend::input::GesturePinchEndEvent<B> + GestureEndEvent<B>,
{
    if state.has_pointer() {
        state.gesture_pinch_end(&GesturePinchEndEvent {
            serial: SERIAL_COUNTER.next_serial(),
            time: event.time_msec(),
            cancelled: event.cancelled(),
        });
    }
}

/// Handles the beginning of a touchpad hold gesture (fingers resting stationary on the pad).
pub(crate) fn handle_hold_begin<B, E>(state: &mut impl InputContext, event: E)
where
    B: InputBackend,
    E: smithay::backend::input::GestureHoldBeginEvent<B> + GestureBeginEvent<B>,
{
    if state.has_pointer() {
        state.gesture_hold_begin(&GestureHoldBeginEvent {
            serial: SERIAL_COUNTER.next_serial(),
            time: event.time_msec(),
            fingers: event.fingers(),
        });
    }
}

/// Handles the end or cancellation of a touchpad hold gesture.
pub(crate) fn handle_hold_end<B, E>(state: &mut impl InputContext, event: E)
where
    B: InputBackend,
    E: smithay::backend::input::GestureHoldEndEvent<B> + GestureEndEvent<B>,
{
    if state.has_pointer() {
        state.gesture_hold_end(&GestureHoldEndEvent {
            serial: SERIAL_COUNTER.next_serial(),
            time: event.time_msec(),
            cancelled: event.cancelled(),
        });
    }
}
