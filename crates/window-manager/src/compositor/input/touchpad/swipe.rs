//! Multi-finger swipe gestures: 3-finger window switching/maximize/minimize,
//! 4-finger window switching, fullscreen, and close.

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
use crate::compositor::state::{BabydraState, SwipeDirection, TrackpadSwipeBuffer};
use services::config::load_gesture_config;

/// Number of fingers for 3-finger gestures (window focus cycling, maximize, minimize).
const THREE_FINGERS: u32 = 3;

/// Number of fingers for 4-finger gestures (focus cycling, fullscreen, close).
const FOUR_FINGERS: u32 = 4;

/// Returns `true` if the given finger count should be captured by the compositor
/// for window management rather than forwarded to clients.
///
/// # Arguments
///
/// * `fingers` - Number of active fingers reported by the gesture event.
#[inline]
fn is_compositor_swipe(fingers: u32) -> bool {
    fingers == THREE_FINGERS || fingers == FOUR_FINGERS
}

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
    E: SwipeBegin<B> + GestureBeginEvent<B>,
{
    let fingers = event.fingers();
    let time = event.time_msec();

    if is_compositor_swipe(fingers) {
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
    E: SwipeEnd<B> + GestureEndEvent<B>,
{
    let time = event.time_msec();
    let cancelled = event.cancelled();

    if let Some(gesture) = state.trackpad_swipe.take() {
        if !cancelled {
            let cfg = load_gesture_config();
            let threshold = if gesture.fingers == FOUR_FINGERS {
                cfg.four_finger_swipe_threshold
            } else {
                cfg.swipe_threshold
            };

            let direction = SwipeDirection::from_delta(
                gesture.total_delta.x,
                gesture.total_delta.y,
                threshold,
            );

            if let Some(dir) = direction {
                let consumed = match gesture.fingers {
                    THREE_FINGERS => handle_three_finger(state, dir, &cfg),
                    FOUR_FINGERS => handle_four_finger(state, dir, &cfg),
                    _ => false,
                };
                if consumed {
                    return;
                }
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

/// Dispatches a 3-finger swipe action based on direction and gesture config.
///
/// Returns `true` when an action was fired (swipe consumed by compositor).
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `dir` - Resolved swipe direction.
/// * `cfg` - Current gesture configuration.
fn handle_three_finger(
    state: &mut BabydraState,
    dir: SwipeDirection,
    cfg: &services::config::GestureConfig,
) -> bool {
    match dir {
        SwipeDirection::Left if cfg.three_finger_swipe_window => {
            cycle_window_focus(state, false, SERIAL_COUNTER.next_serial())
        }
        SwipeDirection::Right if cfg.three_finger_swipe_window => {
            cycle_window_focus(state, true, SERIAL_COUNTER.next_serial())
        }
        SwipeDirection::Up if cfg.three_finger_swipe_up_maximize => {
            if let Some(window) = state.active_window() {
                state.toggle_maximize(&window);
                return true;
            }
            false
        }
        SwipeDirection::Down if cfg.three_finger_swipe_down_minimize => {
            if let Some(window) = state.active_window() {
                state.minimize_window(&window);
                return true;
            }
            false
        }
        _ => false,
    }
}

/// Dispatches a 4-finger swipe action based on direction and gesture config.
///
/// Returns `true` when an action was fired (swipe consumed by compositor).
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `dir` - Resolved swipe direction.
/// * `cfg` - Current gesture configuration.
fn handle_four_finger(
    state: &mut BabydraState,
    dir: SwipeDirection,
    cfg: &services::config::GestureConfig,
) -> bool {
    match dir {
        SwipeDirection::Left if cfg.four_finger_swipe_window => {
            cycle_window_focus(state, false, SERIAL_COUNTER.next_serial())
        }
        SwipeDirection::Right if cfg.four_finger_swipe_window => {
            cycle_window_focus(state, true, SERIAL_COUNTER.next_serial())
        }
        SwipeDirection::Up if cfg.four_finger_swipe_up_fullscreen => {
            if let Some(window) = state.active_window() {
                state.toggle_fullscreen(&window);
                return true;
            }
            false
        }
        SwipeDirection::Down if cfg.four_finger_swipe_down_close => {
            if let Some(window) = state.active_window() {
                state.close_window(&window);
                return true;
            }
            false
        }
        _ => false,
    }
}

/// Replays buffered swipe events down to Wayland clients if not consumed by the compositor.
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
