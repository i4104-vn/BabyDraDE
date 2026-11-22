//! Touchpad pinch and hold gesture event processing.

use smithay::{
    backend::input::{
        GestureBeginEvent, GestureEndEvent, GestureHoldBeginEvent as HoldBegin,
        GestureHoldEndEvent as HoldEnd, GesturePinchBeginEvent as PinchBegin,
        GesturePinchEndEvent as PinchEnd, GesturePinchUpdateEvent as PinchUpdate, InputBackend,
    },
    input::pointer::{
        GestureHoldBeginEvent, GestureHoldEndEvent, GesturePinchBeginEvent, GesturePinchEndEvent,
        GesturePinchUpdateEvent,
    },
    utils::SERIAL_COUNTER,
};

use crate::compositor::state::BabydraState;

/// Handles the start of a pinch gesture on the touchpad.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The pinch begin event providing finger count and timestamp.
pub(crate) fn handle_pinch_begin<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: PinchBegin<B> + GestureBeginEvent<B>,
{
    if let Some(pointer) = state.seat.get_pointer() {
        pointer.gesture_pinch_begin(
            state,
            &GesturePinchBeginEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time: event.time_msec(),
                fingers: event.fingers(),
            },
        );
    }
}

/// Handles incremental scale, rotation, and translation updates during a pinch gesture.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The pinch update event providing delta, scale factor, and rotation angle.
pub(crate) fn handle_pinch_update<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: PinchUpdate<B>,
{
    if let Some(pointer) = state.seat.get_pointer() {
        pointer.gesture_pinch_update(
            state,
            &GesturePinchUpdateEvent {
                time: event.time_msec(),
                delta: event.delta(),
                scale: event.scale(),
                rotation: event.rotation(),
            },
        );
    }
}

/// Handles the completion or cancellation of a touchpad pinch gesture.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The pinch end event indicating cancellation status and timestamp.
pub(crate) fn handle_pinch_end<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: PinchEnd<B> + GestureEndEvent<B>,
{
    if let Some(pointer) = state.seat.get_pointer() {
        pointer.gesture_pinch_end(
            state,
            &GesturePinchEndEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time: event.time_msec(),
                cancelled: event.cancelled(),
            },
        );
    }
}

/// Handles the beginning of a touchpad hold gesture (fingers resting stationary on the pad).
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The hold begin event providing finger count and timestamp.
pub(crate) fn handle_hold_begin<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: HoldBegin<B> + GestureBeginEvent<B>,
{
    if let Some(pointer) = state.seat.get_pointer() {
        pointer.gesture_hold_begin(
            state,
            &GestureHoldBeginEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time: event.time_msec(),
                fingers: event.fingers(),
            },
        );
    }
}

/// Handles the end or cancellation of a touchpad hold gesture.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The hold end event indicating cancellation status and timestamp.
pub(crate) fn handle_hold_end<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: HoldEnd<B> + GestureEndEvent<B>,
{
    if let Some(pointer) = state.seat.get_pointer() {
        pointer.gesture_hold_end(
            state,
            &GestureHoldEndEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time: event.time_msec(),
                cancelled: event.cancelled(),
            },
        );
    }
}
