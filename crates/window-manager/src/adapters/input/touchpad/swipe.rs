//! Multi-finger swipe gestures: 3-finger window switching/maximize/minimize,
//! 4-finger window switching, fullscreen, and close.

use smithay::{
    backend::input::{GestureBeginEvent, GestureEndEvent, InputBackend},
    input::pointer::{GestureSwipeBeginEvent, GestureSwipeEndEvent, GestureSwipeUpdateEvent},
    utils::{Point, SERIAL_COUNTER},
};

use crate::{
    adapters::{
        compositor::state::{BabydraState, TrackpadSwipeBuffer},
        input::execute_window_intent,
    },
    application::gestures::{is_compositor_swipe, should_start_scrub, touchpad_swipe_intent},
};

use common::constants::SWIPE_THREE_FINGERS;

/// Handles the start of a multi-finger swipe gesture.
///
/// Buffers 3- or 4-finger swipes locally so the compositor can decide whether
/// to execute a window management action when the gesture ends.
/// All other finger counts are forwarded to Wayland clients as pointer gesture events.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The swipe begin event providing finger count and timestamp.
pub(crate) fn handle_swipe_begin<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: smithay::backend::input::GestureSwipeBeginEvent<B> + GestureBeginEvent<B>,
{
    let fingers = event.fingers();
    let time = event.time_msec();

    if is_compositor_swipe(fingers) {
        state.trackpad_swipe = Some(TrackpadSwipeBuffer {
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
    E: smithay::backend::input::GestureSwipeUpdateEvent<B>,
{
    let delta = event.delta();
    let time = event.time_msec();

    if state.trackpad_swipe.is_some() {
        {
            let gesture = state.trackpad_swipe.as_mut().unwrap();
            gesture.total_delta.x += delta.x;
            gesture.total_delta.y += delta.y;
            gesture.updates.push((delta, time));
        }

        // 3-finger horizontal drag scrubs the switcher live instead of
        // cycling focus only when the gesture ends.
        let snapshot = state.trackpad_swipe.as_ref().map(|gesture| {
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
                && should_start_scrub(dx, dy, &state.gesture_config)
            {
                if let Some(base_index) = state.begin_switcher_scrub() {
                    let gesture = state.trackpad_swipe.as_mut().unwrap();
                    gesture.scrubbing = true;
                    gesture.scrub_start_x = dx;
                    gesture.scrub_base_index = base_index;
                }
            }

            let scrub = state.trackpad_swipe.as_ref().map(|gesture| {
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

    if let Some(pointer) = state.seat.get_pointer() {
        pointer.gesture_swipe_update(state, &GestureSwipeUpdateEvent { time, delta });
    }
}

/// Handles the termination of a multi-finger swipe gesture.
///
/// Resolves the dominant direction from the accumulated delta and executes the
/// matching window management action when thresholds are met.
///
/// **3-finger actions:**
/// - Left / Right → cycle window focus
/// - Up → maximize active window
/// - Down → minimize active window
///
/// **4-finger actions:**
/// - Left / Right → cycle window focus
/// - Up → toggle fullscreen on active window
/// - Down → close active window
///
/// If the gesture was cancelled, or no threshold was reached, the buffered
/// swipe events are replayed to Wayland clients unchanged.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The swipe end event indicating whether the gesture finished or was cancelled.
pub(crate) fn handle_swipe_end<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: smithay::backend::input::GestureSwipeEndEvent<B> + GestureEndEvent<B>,
{
    let time = event.time_msec();
    let cancelled = event.cancelled();

    match state.trackpad_swipe.take() {
        Some(gesture) => {
            // A live switcher scrub commits the current entry on release and
            // does not replay the swipe to clients.
            if gesture.scrubbing {
                state.commit_switcher_selection();
                return;
            }
            if !cancelled {
                let cfg = state.gesture_config.clone();
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
            if let Some(pointer) = state.seat.get_pointer() {
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
    }
}

/// Replays buffered swipe events down to Wayland clients when the compositor
/// did not consume the gesture.
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
