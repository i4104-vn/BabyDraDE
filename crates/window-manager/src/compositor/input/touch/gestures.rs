//! Multi-touch gesture detection for touchscreens.
//!
//! Tracks per-slot positions to detect:
//! - **2-finger horizontal swipe** → cycle window focus (left / right)
//! - **2-finger swipe down** → minimize active window
//! - **3-finger tap** → send middle-click to focused surface

use smithay::{
    backend::input::TouchSlot,
    input::pointer::ButtonEvent,
    utils::{Point, SERIAL_COUNTER},
};

use crate::compositor::input::keyboard::cycle_window_focus;
use crate::compositor::state::{BabydraState, SwipeDirection, TouchSlotData};
use services::config::load_gesture_config;

// ----- constants ---------------------------------------------------------

/// Linux button code for middle mouse button.
const BTN_MIDDLE: u32 = 0x112;

// ----- down --------------------------------------------------------------

/// Records the start position and time for a newly contacted finger.
///
/// Call this from `handle_touch_down` after the slot is inserted into
/// `state.touch_slots` to begin tracking the finger for gesture detection.
/// Any previously consumed gesture state is reset so a fresh gesture can begin.
///
/// # Arguments
///
/// * `state`    - Mutable reference to [`BabydraState`].
/// * `slot`     - Touch slot index identifying this finger.
/// * `position` - Logical screen position at first contact.
/// * `time_ms`  - Event timestamp in milliseconds.
pub(crate) fn on_touch_down(
    state: &mut BabydraState,
    slot: TouchSlot,
    position: Point<f64, smithay::utils::Logical>,
    time_ms: u64,
) {
    // Reset consumed flag whenever a finger count changes so a new gesture can start.
    state.touch_gesture.consumed = false;
    state.touch_gesture.slots.insert(
        slot,
        TouchSlotData {
            start: position,
            current: position,
            start_time_ms: time_ms,
        },
    );
}

// ----- motion ------------------------------------------------------------

/// Updates the current position of an active touch slot during finger movement.
///
/// Should be called from `handle_touch_motion` for multi-touch gesture tracking.
///
/// # Arguments
///
/// * `state`    - Mutable reference to [`BabydraState`].
/// * `slot`     - The touch slot index being updated.
/// * `position` - Updated logical screen position for this slot.
pub(crate) fn on_touch_motion(
    state: &mut BabydraState,
    slot: TouchSlot,
    position: Point<f64, smithay::utils::Logical>,
) {
    if let Some(data) = state.touch_gesture.slots.get_mut(&slot) {
        data.current = position;
    }
}

// ----- up ----------------------------------------------------------------

/// Evaluates accumulated gesture data when a finger lifts off.
///
/// If the gesture has not yet been consumed, checks whether the active slots
/// formed a recognisable pattern (2-finger swipe or 3-finger tap) and fires the
/// corresponding compositor action.
///
/// Call this from `handle_touch_up` **before** removing the slot from
/// `state.touch_slots` so the finger count is still correct during evaluation.
///
/// # Arguments
///
/// * `state`   - Mutable reference to [`BabydraState`].
/// * `slot`    - The touch slot that just lifted.
/// * `time_ms` - Current event timestamp in milliseconds.
pub(crate) fn on_touch_up(state: &mut BabydraState, slot: TouchSlot, time_ms: u64) {
    let n = state.touch_gesture.slots.len();
    let consumed = state.touch_gesture.consumed;

    if !consumed && n >= 2 {
        let cfg = load_gesture_config();
        let _ = try_fire_touch_gesture(state, n, time_ms, &cfg);
    }

    state.touch_gesture.slots.remove(&slot);

    // Reset consumed flag once all fingers are off the screen.
    if state.touch_gesture.slots.is_empty() {
        state.touch_gesture.consumed = false;
    }
}

// ----- gesture evaluation ------------------------------------------------

/// Attempts to recognise and fire a multi-touch gesture action.
///
/// Inspects the accumulated slot data to determine whether the fingers formed a
/// 2-finger swipe or a 3-finger tap. If a gesture is recognised, the corresponding
/// compositor action is executed and `state.touch_gesture.consumed` is set to `true`.
///
/// # Arguments
///
/// * `state`   - Mutable reference to [`BabydraState`].
/// * `n`       - Number of currently tracked slots (finger count at gesture end).
/// * `time_ms` - Current event timestamp in milliseconds.
/// * `cfg`     - Current gesture configuration loaded from `babydra.conf`.
///
/// # Returns
///
/// `true` if a gesture was matched and fired, `false` otherwise.
fn try_fire_touch_gesture(
    state: &mut BabydraState,
    n: usize,
    time_ms: u64,
    cfg: &services::config::GestureConfig,
) -> bool {
    match n {
        2 => try_two_finger_swipe(state, cfg),
        3 => try_three_finger_tap(state, time_ms, cfg),
        _ => false,
    }
}

/// Checks whether the two tracked slots form a horizontal or downward swipe and fires it.
///
/// Both fingers must have moved in the same dominant direction by at least
/// `cfg.touch_swipe_threshold` pixels.
/// - Horizontal (left / right) → cycle window focus.
/// - Down → minimize the active window.
///
/// # Arguments
///
/// * `state`  - Mutable reference to [`BabydraState`].
/// * `cfg`    - Gesture configuration containing thresholds and feature flags.
///
/// # Returns
///
/// `true` if the swipe was recognised and an action was fired.
fn try_two_finger_swipe(
    state: &mut BabydraState,
    cfg: &services::config::GestureConfig,
) -> bool {
    let threshold = cfg.touch_swipe_threshold;
    let slots: Vec<_> = state.touch_gesture.slots.values().cloned().collect();
    if slots.len() < 2 {
        return false;
    }

    // Compute per-finger delta.
    let d0 = slots[0].current - slots[0].start;
    let d1 = slots[1].current - slots[1].start;

    // Both fingers must agree on the dominant axis and direction.
    let dir0 = SwipeDirection::from_delta(d0.x, d0.y, threshold);
    let dir1 = SwipeDirection::from_delta(d1.x, d1.y, threshold);

    if dir0.is_none() || dir0 != dir1 {
        return false;
    }

    let dir = dir0.unwrap();
    let fired = match dir {
        SwipeDirection::Left if cfg.touch_two_finger_swipe_window => {
            cycle_window_focus(state, false, SERIAL_COUNTER.next_serial())
        }
        SwipeDirection::Right if cfg.touch_two_finger_swipe_window => {
            cycle_window_focus(state, true, SERIAL_COUNTER.next_serial())
        }
        SwipeDirection::Down if cfg.touch_two_finger_swipe_down_minimize => {
            if let Some(window) = state.active_window() {
                state.minimize_window(&window);
                true
            } else {
                false
            }
        }
        _ => false,
    };

    if fired {
        state.touch_gesture.consumed = true;
    }
    fired
}

/// Checks whether three tracked slots form a tap (short duration, little movement) and
/// synthesises a middle-click pointer event if so.
///
/// A tap is defined as: all three fingers lifted within `cfg.touch_tap_timeout_ms` ms of
/// first contact and each finger moved fewer than 20 pixels.
///
/// # Arguments
///
/// * `state`   - Mutable reference to [`BabydraState`].
/// * `time_ms` - Timestamp of the lift event in milliseconds.
/// * `cfg`     - Gesture configuration containing tap timeout and feature flags.
///
/// # Returns
///
/// `true` if a tap was recognised and a middle-click was synthesised.
fn try_three_finger_tap(
    state: &mut BabydraState,
    time_ms: u64,
    cfg: &services::config::GestureConfig,
) -> bool {
    if !cfg.touch_three_finger_tap {
        return false;
    }

    let slots: Vec<_> = state.touch_gesture.slots.values().cloned().collect();
    if slots.len() < 3 {
        return false;
    }

    // All fingers must have tapped quickly and not moved much.
    let all_tap = slots.iter().all(|s| {
        let elapsed = time_ms.saturating_sub(s.start_time_ms);
        let d = s.current - s.start;
        let moved = d.x.hypot(d.y);
        elapsed <= cfg.touch_tap_timeout_ms && moved < 20.0
    });

    if !all_tap {
        return false;
    }

    // Synthesise a middle-click at the pointer's current position.
    if let Some(pointer) = state.seat.get_pointer() {
        let serial = SERIAL_COUNTER.next_serial();
        pointer.button(
            state,
            &ButtonEvent {
                serial,
                time: time_ms as u32,
                button: BTN_MIDDLE,
                state: smithay::backend::input::ButtonState::Pressed,
            },
        );
        let serial2 = SERIAL_COUNTER.next_serial();
        pointer.button(
            state,
            &ButtonEvent {
                serial: serial2,
                time: time_ms as u32,
                button: BTN_MIDDLE,
                state: smithay::backend::input::ButtonState::Released,
            },
        );
        state.touch_gesture.consumed = true;
        return true;
    }

    false
}
