//! Use case: turn raw multi-finger motion into a window-management intent.
//!
//! The rules live here, free of Smithay events and compositor state. Adapters
//! translate hardware events into the small inputs below and execute the
//! returned [`WindowIntent`] against the compositor.

use common::SwipeDirection;

use super::ports::GesturePolicy;

/// A decision the core has made; the adapter decides how to perform it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WindowIntent {
    /// Swallow the event without further action (already handled live).
    Consumed,
    CycleFocus {
        forward: bool,
    },
    Maximize,
    Minimize,
    Fullscreen,
    Close,
    /// Synthesise a middle click.
    MiddleClick,
}

/// Distance (logical px) after which a horizontal drag starts scrubbing.
const SCRUB_START_DISTANCE: f64 = 40.0;

/// Reports whether a swipe must be captured by the compositor.
///
/// # Arguments
///
/// * `fingers` - Number of active fingers reported by the backend.
///
/// # Returns
///
/// `true` for three- and four-finger swipes.
pub(crate) fn is_compositor_swipe(fingers: u32) -> bool {
    fingers == 3 || fingers == 4
}

/// Decides whether a drag should start live switcher scrubbing.
///
/// # Arguments
///
/// * `dx` - Accumulated horizontal delta in logical pixels.
/// * `dy` - Accumulated vertical delta in logical pixels.
/// * `cfg` - Active gesture policy.
///
/// # Returns
///
/// `true` when the drag is horizontal far enough to start scrubbing.
pub(crate) fn should_start_scrub(dx: f64, dy: f64, cfg: &GesturePolicy) -> bool {
    cfg.three_finger_swipe_window && dx.abs() >= SCRUB_START_DISTANCE && dx.abs() > dy.abs()
}

/// Resolves a completed touchpad swipe into a window action.
///
/// # Arguments
///
/// * `fingers` - Number of fingers in the swipe.
/// * `delta` - Accumulated `(dx, dy)` in logical pixels.
/// * `cfg` - Active gesture policy.
///
/// # Returns
///
/// The matching intent, or `None` when the threshold is not met or the
/// gesture is disabled.
pub(crate) fn touchpad_swipe_intent(
    fingers: u32,
    delta: (f64, f64),
    cfg: &GesturePolicy,
) -> Option<WindowIntent> {
    let threshold = if fingers == 4 {
        cfg.four_finger_swipe_threshold
    } else {
        cfg.swipe_threshold
    };
    let direction = SwipeDirection::from_delta(delta.0, delta.1, threshold)?;
    match (fingers, direction) {
        (3, SwipeDirection::Left | SwipeDirection::Right) if cfg.three_finger_swipe_window => {
            Some(WindowIntent::Consumed)
        }
        (3, SwipeDirection::Up) if cfg.three_finger_swipe_up_maximize => {
            Some(WindowIntent::Maximize)
        }
        (3, SwipeDirection::Down) if cfg.three_finger_swipe_down_minimize => {
            Some(WindowIntent::Minimize)
        }
        (4, SwipeDirection::Left) if cfg.four_finger_swipe_window => {
            Some(WindowIntent::CycleFocus { forward: false })
        }
        (4, SwipeDirection::Right) if cfg.four_finger_swipe_window => {
            Some(WindowIntent::CycleFocus { forward: true })
        }
        (4, SwipeDirection::Up) if cfg.four_finger_swipe_up_fullscreen => {
            Some(WindowIntent::Fullscreen)
        }
        (4, SwipeDirection::Down) if cfg.four_finger_swipe_down_close => Some(WindowIntent::Close),
        _ => None,
    }
}

/// Resolves a two-finger touchscreen swipe.
///
/// # Arguments
///
/// * `deltas` - Per-finger `(dx, dy)` movement since touch-down.
/// * `cfg` - Active gesture policy.
///
/// # Returns
///
/// The matching intent, or `None` if the fingers disagree or no rule matches.
pub(crate) fn two_finger_touch_intent(
    deltas: [(f64, f64); 2],
    cfg: &GesturePolicy,
) -> Option<WindowIntent> {
    let first = SwipeDirection::from_delta(deltas[0].0, deltas[0].1, cfg.touch_swipe_threshold)?;
    let second = SwipeDirection::from_delta(deltas[1].0, deltas[1].1, cfg.touch_swipe_threshold)?;
    if first != second {
        return None;
    }
    match first {
        SwipeDirection::Left if cfg.touch_two_finger_swipe_window => {
            Some(WindowIntent::CycleFocus { forward: false })
        }
        SwipeDirection::Right if cfg.touch_two_finger_swipe_window => {
            Some(WindowIntent::CycleFocus { forward: true })
        }
        SwipeDirection::Down if cfg.touch_two_finger_swipe_down_minimize => {
            Some(WindowIntent::Minimize)
        }
        _ => None,
    }
}

/// Resolves a three-finger tap.
///
/// # Arguments
///
/// * `elapsed_ms` - Milliseconds since the first finger touched down.
/// * `moved` - Distance the finger travelled in logical pixels.
/// * `cfg` - Active gesture policy.
///
/// # Returns
///
/// `MiddleClick` for a short, stationary tap; `None` otherwise.
pub(crate) fn three_finger_tap_intent(
    elapsed_ms: u64,
    moved: f64,
    cfg: &GesturePolicy,
) -> Option<WindowIntent> {
    if cfg.touch_three_finger_tap && elapsed_ms <= cfg.touch_tap_timeout_ms && moved < 20.0 {
        Some(WindowIntent::MiddleClick)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> GesturePolicy {
        GesturePolicy {
            swipe_threshold: 100.0,
            four_finger_swipe_threshold: 120.0,
            touch_swipe_threshold: 80.0,
            touch_tap_timeout_ms: 200,
            three_finger_swipe_window: true,
            three_finger_swipe_up_maximize: true,
            three_finger_swipe_down_minimize: true,
            four_finger_swipe_window: true,
            four_finger_swipe_up_fullscreen: true,
            four_finger_swipe_down_close: true,
            touch_two_finger_swipe_window: true,
            touch_two_finger_swipe_down_minimize: true,
            touch_three_finger_tap: true,
        }
    }

    #[test]
    fn only_three_and_four_finger_swipes_are_compositor_owned() {
        assert!(!is_compositor_swipe(1));
        assert!(!is_compositor_swipe(2));
        assert!(is_compositor_swipe(3));
        assert!(is_compositor_swipe(4));
        assert!(!is_compositor_swipe(5));
    }

    #[test]
    fn touchpad_swipes_map_to_window_intents() {
        assert_eq!(
            touchpad_swipe_intent(3, (0.0, -150.0), &policy()),
            Some(WindowIntent::Maximize)
        );
        assert_eq!(
            touchpad_swipe_intent(4, (150.0, 0.0), &policy()),
            Some(WindowIntent::CycleFocus { forward: true })
        );
        assert_eq!(touchpad_swipe_intent(3, (10.0, 0.0), &policy()), None);
    }

    #[test]
    fn touch_swipes_require_both_fingers_to_agree() {
        assert_eq!(
            two_finger_touch_intent([(90.0, 0.0), (85.0, 5.0)], &policy()),
            Some(WindowIntent::CycleFocus { forward: true })
        );
        assert_eq!(
            two_finger_touch_intent([(90.0, 0.0), (-85.0, 0.0)], &policy()),
            None
        );
    }
}
