use smithay::{
    backend::input::{
        Axis, AxisSource, GestureBeginEvent, GestureEndEvent, GestureHoldBeginEvent as HoldBegin,
        GestureHoldEndEvent as HoldEnd, GesturePinchBeginEvent as PinchBegin,
        GesturePinchEndEvent as PinchEnd, GesturePinchUpdateEvent as PinchUpdate,
        GestureSwipeBeginEvent as SwipeBegin, GestureSwipeEndEvent as SwipeEnd,
        GestureSwipeUpdateEvent as SwipeUpdate, InputBackend, PointerAxisEvent,
    },
    input::pointer::{
        AxisFrame, GestureHoldBeginEvent, GestureHoldEndEvent, GesturePinchBeginEvent,
        GesturePinchEndEvent, GesturePinchUpdateEvent, GestureSwipeBeginEvent,
        GestureSwipeEndEvent, GestureSwipeUpdateEvent,
    },
    utils::{Point, SERIAL_COUNTER},
};

use super::super::state::{BabydraState, TrackpadSwipeBuffer};
use super::keyboard::cycle_window_focus;

const WINDOW_SWITCH_FINGERS: u32 = 3;
const WINDOW_SWITCH_THRESHOLD: f64 = 100.0;

pub(crate) fn handle_axis<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: PointerAxisEvent<B>,
{
    let source = event.source();
    let mut frame = AxisFrame::new(event.time_msec())
        .source(source)
        .relative_direction(Axis::Horizontal, event.relative_direction(Axis::Horizontal))
        .relative_direction(Axis::Vertical, event.relative_direction(Axis::Vertical));

    for axis in [Axis::Horizontal, Axis::Vertical] {
        let amount = event.amount(axis);
        let amount_v120 = event.amount_v120(axis);

        if let Some(amount) = amount {
            if source == AxisSource::Finger && amount == 0.0 {
                frame = frame.stop(axis);
            } else {
                frame = frame.value(axis, amount);
            }
        } else if let Some(amount_v120) = amount_v120 {
            // Provide a legacy pixel value as well as v120 for wheel-only backends.
            frame = frame.value(axis, amount_v120 / 8.0);
        }

        if let Some(amount_v120) = amount_v120 {
            frame = frame.v120(axis, amount_v120.round() as i32);
        }
    }

    if let Some(pointer) = state.seat.get_pointer() {
        pointer.axis(state, frame);
        pointer.frame(state);
    }
}

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
