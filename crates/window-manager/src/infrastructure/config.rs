//! Configuration adapter — translates services config models into core policies.

use crate::application::ports::GesturePolicy;

/// Converts the persisted gesture model into the framework-free policy.
///
/// # Arguments
///
/// * `cfg` - Gesture configuration loaded from disk.
///
/// # Returns
///
/// The policy snapshot consumed by the gesture use case.
impl From<&services::models::GestureConfig> for GesturePolicy {
    fn from(cfg: &services::models::GestureConfig) -> Self {
        Self {
            swipe_threshold: cfg.swipe_threshold,
            four_finger_swipe_threshold: cfg.four_finger_swipe_threshold,
            touch_swipe_threshold: cfg.touch_swipe_threshold,
            touch_tap_timeout_ms: cfg.touch_tap_timeout_ms,
            three_finger_swipe_window: cfg.three_finger_swipe_window,
            three_finger_swipe_up_maximize: cfg.three_finger_swipe_up_maximize,
            three_finger_swipe_down_minimize: cfg.three_finger_swipe_down_minimize,
            four_finger_swipe_window: cfg.four_finger_swipe_window,
            four_finger_swipe_up_fullscreen: cfg.four_finger_swipe_up_fullscreen,
            four_finger_swipe_down_close: cfg.four_finger_swipe_down_close,
            touch_two_finger_swipe_window: cfg.touch_two_finger_swipe_window,
            touch_two_finger_swipe_down_minimize: cfg.touch_two_finger_swipe_down_minimize,
            touch_three_finger_tap: cfg.touch_three_finger_tap,
        }
    }
}

/// Reloads gesture settings from disk.
///
/// # Returns
///
/// A fresh policy snapshot for the compositor state.
pub(crate) fn gesture_policy() -> GesturePolicy {
    GesturePolicy::from(&services::config::load_gesture_config())
}
