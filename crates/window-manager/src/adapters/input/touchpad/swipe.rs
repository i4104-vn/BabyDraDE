//! Multi-finger swipe gestures: 3-finger window switching/maximize/minimize,
//! 4-finger window switching, fullscreen, and close.

use smithay::{
    backend::input::{GestureBeginEvent, GestureEndEvent, InputBackend},
    input::pointer::{GestureSwipeBeginEvent, GestureSwipeEndEvent, GestureSwipeUpdateEvent},
    utils::{Point, SERIAL_COUNTER},
};

use super::super::InputContext;
use crate::adapters::input::{execute_window_intent, TrackpadSwipeBuffer};
use crate::application::gestures::{is_compositor_swipe, should_start_scrub, touchpad_swipe_intent};

use common::constants::SWIPE_THREE_FINGERS;

/// Handles the start of a multi-finger swipe gesture.
pub(crate) fn handle_swipe_begin<B, E>(state: &mut impl InputContext, event: E)
where
    B: InputBackend,
    E: smithay::backend::input::GestureSwipeBeginEvent<B> + GestureBeginEvent<B>,
{
    let fingers = event.fingers();
    let time = event.time_msec();

    if is_compositor_swipe(fingers) {
        *state.trackpad_swipe_mut() = Some(TrackpadSwipeBuffer {
            fingers,
            time,
            total_delta: Point::from((0.0, 0.0)),
            updates: Vec::new(),
            scrubbing: false,
            scrub_start_x: 0.0,
            scrub_base_index: 0,
        });
        return;
    }

    if state.has_pointer() {
        state.gesture_swipe_begin(&GestureSwipeBeginEvent {
            serial: SERIAL_COUNTER.next_serial(),
            time,
            fingers,
        });
    }
}

/// Accumulates motion delta updates during an active multi-finger swipe.
pub(crate) fn handle_swipe_update<B, E>(state: &mut impl InputContext, event: E)
where
    B: InputBackend,
    E: smithay::backend::input::GestureSwipeUpdateEvent<B>,
{
    let delta = event.delta();
    let time = event.time_msec();

    if state.trackpad_swipe().is_some() {
        {
            let gesture = state.trackpad_swipe_mut().as_mut().unwrap();
            gesture.total_delta.x += delta.x;
            gesture.total_delta.y += delta.y;
            gesture.updates.push((delta, time));
        }

        // 3-finger horizontal drag scrubs the switcher live instead of
        // cycling focus only when the gesture ends.
        let snapshot = state.trackpad_swipe().map(|gesture| {
            (
                gesture.fingers,
                gesture.scrubbing,
                gesture.total_delta.x,
                gesture.total_delta.y,
            )
        });
        if let Some((fingers, scrubbing, dx, dy)) = snapshot {
            if fingers == SWIPE_THREE_FINGERS
                && !scrubbing
                && should_start_scrub(dx, dy, state.gesture_config())
            {
                if let Some(base_index) = state.begin_switcher_scrub() {
                    let gesture = state.trackpad_swipe_mut().as_mut().unwrap();
                    gesture.scrubbing = true;
                    gesture.scrub_start_x = dx;
                    gesture.scrub_base_index = base_index;
                }
            }

            let scrub = state.trackpad_swipe().map(|gesture| {
                (
                    gesture.scrubbing,
                    gesture.scrub_start_x,
                    gesture.scrub_base_index,
                    dx,
                )
            });
            if let Some((true, start_x, base_index, current_x)) = scrub {
                state.scrub_switcher(start_x, base_index, current_x);
            }
        }
        return;
    }

    if state.has_pointer() {
        state.gesture_swipe_update(&GestureSwipeUpdateEvent { time, delta });
    }
}

/// Handles the termination of a multi-finger swipe gesture.
pub(crate) fn handle_swipe_end<B, E>(state: &mut impl InputContext, event: E)
where
    B: InputBackend,
    E: smithay::backend::input::GestureSwipeEndEvent<B> + GestureEndEvent<B>,
{
    let time = event.time_msec();
    let cancelled = event.cancelled();

    match state.trackpad_swipe_mut().take() {
        Some(gesture) => {
            // A live switcher scrub commits the current entry on release and
            // does not replay the swipe to clients.
            if gesture.scrubbing {
                state.commit_switcher_selection();
                return;
            }
            if !cancelled {
                let cfg = state.gesture_config().clone();
                let intent = touchpad_swipe_intent(
                    gesture.fingers,
                    (gesture.total_delta.x, gesture.total_delta.y),
                    &cfg,
                );
                if let Some(intent) = intent {
                    if execute_window_intent(state, intent) {
                        return;
                    }
                }
            }

            replay_swipe(state, gesture, cancelled, time);
        }
        None => {
            if state.has_pointer() {
                state.gesture_swipe_end(&GestureSwipeEndEvent {
                    serial: SERIAL_COUNTER.next_serial(),
                    time,
                    cancelled,
                });
            }
        }
    }
}

/// Replays buffered swipe events down to Wayland clients when the compositor
/// did not consume the gesture.
fn replay_swipe<C: InputContext>(
    state: &mut C,
    gesture: TrackpadSwipeBuffer,
    cancelled: bool,
    end_time: u32,
) {
    if !state.has_pointer() {
        return;
    }

    state.gesture_swipe_begin(&GestureSwipeBeginEvent {
        serial: SERIAL_COUNTER.next_serial(),
        time: gesture.time,
        fingers: gesture.fingers,
    });
    for (delta, time) in gesture.updates {
        state.gesture_swipe_update(&GestureSwipeUpdateEvent { time, delta });
    }
    state.gesture_swipe_end(&GestureSwipeEndEvent {
        serial: SERIAL_COUNTER.next_serial(),
        time: end_time,
        cancelled,
    });
}
