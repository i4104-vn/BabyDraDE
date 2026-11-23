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
