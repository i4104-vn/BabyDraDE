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

use crate::modules::input::InputContext;
use crate::modules::input::{execute_window_intent, TouchSlotData};
use crate::{
    modules::input::gestures::{
        should_start_scrub, three_finger_tap_intent, two_finger_touch_intent, WindowIntent,
    },
    shared::ports::GesturePolicy,
};

use common::constants::BTN_MIDDLE;

/// Records the start position and time for a newly contacted finger.
pub(crate) fn on_touch_down<C: InputContext>(
    state: &mut C,
    slot: TouchSlot,
    position: Point<f64, smithay::utils::Logical>,
    time_ms: u64,
) {
    // Reset consumed flag whenever a finger count changes so a new gesture can start.
    state.touch_gesture_mut().consumed = false;
    state.touch_gesture_mut().scrubbing = false;
    state.touch_gesture_mut().slots.insert(
        slot,
        TouchSlotData {
            start: position,
            current: position,
            start_time_ms: time_ms,
        },
    );
}

/// Updates the current position of an active touch slot during finger movement.
pub(crate) fn on_touch_motion<C: InputContext>(
    state: &mut C,
    slot: TouchSlot,
    position: Point<f64, smithay::utils::Logical>,
) {
    if let Some(data) = state.touch_gesture_mut().slots.get_mut(&slot) {
        data.current = position;
    }

    // 3-finger horizontal drag: show the switcher and scrub its active entry
    // live with the fingers instead of switching focus at gesture end.
    let snapshot: Vec<_> = state
        .touch_gesture()
        .slots
        .values()
        .map(|data| (data.start, data.current))
        .collect();
    if snapshot.len() >= 3 {
        let (start_x, current_x, start_y, current_y) = {
            let first = &snapshot[..3];
            (
                first.iter().map(|v| v.0.x).sum::<f64>() / 3.0,
                first.iter().map(|v| v.1.x).sum::<f64>() / 3.0,
                first.iter().map(|v| v.0.y).sum::<f64>() / 3.0,
                first.iter().map(|v| v.1.y).sum::<f64>() / 3.0,
            )
        };
        let dx = current_x - start_x;
        let dy = current_y - start_y;
        let scrubbing = state.touch_gesture().scrubbing;
        if !scrubbing && should_start_scrub(dx, dy, state.gesture_config()) {
            if let Some(base_index) = state.begin_switcher_scrub() {
                state.touch_gesture_mut().scrubbing = true;
                state.touch_gesture_mut().scrub_start_x = start_x;
                state.touch_gesture_mut().scrub_base_index = base_index;
            }
        }
        if state.touch_gesture().scrubbing {
            let start_x = state.touch_gesture().scrub_start_x;
            let base_index = state.touch_gesture().scrub_base_index;
            state.scrub_switcher(start_x, base_index, current_x);
        }
    }
}

/// Evaluates accumulated gesture data when a finger lifts off.
pub(crate) fn on_touch_up<C: InputContext>(state: &mut C, slot: TouchSlot, time_ms: u64) {
    let n = state.touch_gesture().slots.len();
    let scrubbing = state.touch_gesture().scrubbing;
    if scrubbing {
        // Releasing the fingers commits the entry currently under them.
        state.commit_switcher_selection();
    }
    let consumed = state.touch_gesture().consumed || scrubbing;

    if !consumed && n >= 2 {
        let cfg = state.gesture_config().clone();
        let _ = try_fire_touch_gesture(state, n, time_ms, &cfg);
    }

    state.touch_gesture_mut().slots.remove(&slot);

    // Reset consumed flag once all fingers are off the screen. The switcher
    // stays open on the last active entry.
    if state.touch_gesture().slots.is_empty() {
        state.touch_gesture_mut().consumed = false;
        state.touch_gesture_mut().scrubbing = false;
    }
}

/// Asks the gesture use case for an intent and executes it against the state.
fn try_fire_touch_gesture<C: InputContext>(
    state: &mut C,
    finger_count: usize,
    time_ms: u64,
    cfg: &GesturePolicy,
) -> bool {
    let intent = match finger_count {
        2 => {
            let slots: Vec<_> = state.touch_gesture().slots.values().cloned().collect();
            if slots.len() < 2 {
                return false;
            }
            let first = slots[0].current - slots[0].start;
            let second = slots[1].current - slots[1].start;
            two_finger_touch_intent([(first.x, first.y), (second.x, second.y)], cfg)
        }
        3 => {
            let slots: Vec<_> = state.touch_gesture().slots.values().cloned().collect();
            if slots.len() < 3 {
                return false;
            }
            let all_tap = slots.iter().all(|slot| {
                let elapsed = time_ms.saturating_sub(slot.start_time_ms);
                let delta = slot.current - slot.start;
                three_finger_tap_intent(elapsed, delta.x.hypot(delta.y), cfg).is_some()
            });
            if all_tap {
                Some(WindowIntent::MiddleClick)
            } else {
                None
            }
        }
        _ => None,
    };

    let Some(intent) = intent else {
        return false;
    };
    let fired = if intent == WindowIntent::MiddleClick {
        synthesize_middle_click(state, time_ms)
    } else {
        execute_window_intent(state, intent)
    };
    if fired {
        state.touch_gesture_mut().consumed = true;
    }
    fired
}

/// Synthesizes the middle click a three-finger tap stands for.
fn synthesize_middle_click<C: InputContext>(state: &mut C, time_ms: u64) -> bool {
    if !state.has_pointer() {
        return false;
    }
    state.pointer_button(&ButtonEvent {
        serial: SERIAL_COUNTER.next_serial(),
        time: time_ms as u32,
        button: BTN_MIDDLE,
        state: smithay::backend::input::ButtonState::Pressed,
    });
    state.pointer_button(&ButtonEvent {
        serial: SERIAL_COUNTER.next_serial(),
        time: time_ms as u32,
        button: BTN_MIDDLE,
        state: smithay::backend::input::ButtonState::Released,
    });
    true
}
