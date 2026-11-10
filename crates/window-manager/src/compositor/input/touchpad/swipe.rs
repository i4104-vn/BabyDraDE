//! Multi-finger swipe gestures and workspace/window switching.

use smithay::{
    backend::input::{
        GestureBeginEvent, GestureEndEvent, GestureSwipeBeginEvent as SwipeBegin,
        GestureSwipeEndEvent as SwipeEnd, GestureSwipeUpdateEvent as SwipeUpdate, InputBackend,
    },
    input::pointer::{
        GestureSwipeBeginEvent, GestureSwipeEndEvent, GestureSwipeUpdateEvent,
    },
    utils::{Point, SERIAL_COUNTER},
};

use crate::compositor::input::keyboard::cycle_window_focus;
use crate::compositor::state::{BabydraState, TrackpadSwipeBuffer};

/// Number of fingers required to trigger window switching gestures.
const WINDOW_SWITCH_FINGERS: u32 = 3;

/// Movement threshold in pixels required before a 3-finger horizontal swipe triggers window cycle.
const WINDOW_SWITCH_THRESHOLD: f64 = 100.0;

/// Handles the start of a multi-finger swipe gesture.
///
/// If a 3-finger swipe starts while multiple windows exist, buffers the swipe delta
/// locally in state to detect window-switching gesture. Otherwise, forwards to pointer clients.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The swipe begin event providing finger count and timestamp.
pub(crate) fn handle_swipe_begin<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: SwipeBegin<B> + GestureBeginEvent<B>,
{
    let fingers = event.fingers();
    let time = event.time_msec();

    if fingers == WINDOW_SWITCH_FINGERS && state.space.elements().count() > 1 {
        state.trackpad_swipe = Some(TrackpadSwipeBuffer {
            fingers,
            time,
            total_delta: Point::from((0.0, 0.0)),
            updates: Vec::new(),
        });
        return;
    }

    if let Some(pointer) = state.seat.get_pointer() {
        pointer.gesture_swipe_begin(
            state,
            &GestureSwipeBeginEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time,
                fingers,
            },
        );
    }
}

/// Accumulates motion delta updates during an active multi-finger swipe.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The swipe update event providing delta x and delta y.
pub(crate) fn handle_swipe_update<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: SwipeUpdate<B>,
{
    let delta = event.delta();
    let time = event.time_msec();

    if let Some(gesture) = state.trackpad_swipe.as_mut() {
        gesture.total_delta.x += delta.x;
        gesture.total_delta.y += delta.y;
        gesture.updates.push((delta, time));
        return;
    }

    if let Some(pointer) = state.seat.get_pointer() {
        pointer.gesture_swipe_update(state, &GestureSwipeUpdateEvent { time, delta });
    }
}

/// Handles the termination of a multi-finger swipe gesture.
///
/// If a 3-finger swipe exceeded [`WINDOW_SWITCH_THRESHOLD`], cycles focus between windows.
/// If the gesture was cancelled or did not meet threshold, replays the buffered swipe events to clients.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The swipe end event indicating whether the gesture finished or was cancelled.
pub(crate) fn handle_swipe_end<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: SwipeEnd<B> + GestureEndEvent<B>,
{
    let time = event.time_msec();
    let cancelled = event.cancelled();

    if let Some(gesture) = state.trackpad_swipe.take() {
        let horizontal_swipe = gesture.total_delta.x.abs() >= WINDOW_SWITCH_THRESHOLD
            && gesture.total_delta.x.abs() > gesture.total_delta.y.abs();

        if !cancelled && horizontal_swipe {
            let forward = gesture.total_delta.x < 0.0;
            if cycle_window_focus(state, forward, SERIAL_COUNTER.next_serial()) {
                return;
            }
        }

        replay_swipe(state, gesture, cancelled, time);
    } else if let Some(pointer) = state.seat.get_pointer() {
        pointer.gesture_swipe_end(
            state,
            &GestureSwipeEndEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time,
                cancelled,
            },
        );
    }
}

/// Replays buffered 3-finger swipe events down to Wayland clients if not consumed by the compositor.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `gesture` - The buffered swipe updates containing start time, finger count, and incremental deltas.
/// * `cancelled` - Whether the swipe was cancelled.
/// * `end_time` - Timestamp in milliseconds of the end event.
fn replay_swipe(
    state: &mut BabydraState,
    gesture: TrackpadSwipeBuffer,
    cancelled: bool,
    end_time: u32,
) {
    let Some(pointer) = state.seat.get_pointer() else {
        return;
    };

    pointer.gesture_swipe_begin(
        state,
        &GestureSwipeBeginEvent {
            serial: SERIAL_COUNTER.next_serial(),
            time: gesture.time,
            fingers: gesture.fingers,
        },
    );
    for (delta, time) in gesture.updates {
        pointer.gesture_swipe_update(state, &GestureSwipeUpdateEvent { time, delta });
    }
    pointer.gesture_swipe_end(
        state,
        &GestureSwipeEndEvent {
            serial: SERIAL_COUNTER.next_serial(),
            time: end_time,
            cancelled,
        },
    );
}

