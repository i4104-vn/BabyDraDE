//! Multi-finger touchpad and touchscreen gesture configuration models.

use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_swipe_threshold() -> f64 {
    100.0
}

fn default_four_finger_swipe_threshold() -> f64 {
    120.0
}

fn default_touch_swipe_threshold() -> f64 {
    80.0
}

fn default_touch_tap_timeout_ms() -> u64 {
    200
}

/// Gesture configuration persisted in `~/.babydra/babydra.conf` under `[gestures]`.
///
/// Controls which multi-finger swipe and touch gestures are active, and
/// their activation distance thresholds.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GestureConfig {
    /// Enable 3-finger horizontal swipe to cycle window focus.
    #[serde(default = "default_true")]
    pub three_finger_swipe_window: bool,
    /// Enable 3-finger swipe up to maximize the active window.
    #[serde(default = "default_true")]
    pub three_finger_swipe_up_maximize: bool,
    /// Enable 3-finger swipe down to minimize the active window.
    #[serde(default = "default_true")]
    pub three_finger_swipe_down_minimize: bool,
    /// Enable 4-finger horizontal swipe to cycle window focus.
    #[serde(default = "default_true")]
    pub four_finger_swipe_window: bool,
    /// Enable 4-finger swipe down to close the active window.
    #[serde(default = "default_true")]
    pub four_finger_swipe_down_close: bool,
    /// Enable 4-finger swipe up to toggle fullscreen on the active window.
    #[serde(default = "default_true")]
    pub four_finger_swipe_up_fullscreen: bool,
    /// Minimum pixel distance for a 3-finger swipe to activate (default 100.0).
    #[serde(default = "default_swipe_threshold")]
    pub swipe_threshold: f64,
    /// Minimum pixel distance for a 4-finger swipe to activate (default 120.0).
    #[serde(default = "default_four_finger_swipe_threshold")]
    pub four_finger_swipe_threshold: f64,
    /// Enable 2-finger touch swipe left/right to cycle window focus on touchscreens.
    #[serde(default = "default_true")]
    pub touch_two_finger_swipe_window: bool,
    /// Enable 2-finger touch swipe down to minimize the active window.
    #[serde(default = "default_true")]
    pub touch_two_finger_swipe_down_minimize: bool,
    /// Enable 3-finger touch tap to send a middle-click.
    #[serde(default = "default_true")]
    pub touch_three_finger_tap: bool,
    /// Minimum pixel distance for a 2-finger touch swipe to activate (default 80.0).
    #[serde(default = "default_touch_swipe_threshold")]
    pub touch_swipe_threshold: f64,
    /// Maximum time in milliseconds from first contact to lift for a tap (default 200).
    #[serde(default = "default_touch_tap_timeout_ms")]
    pub touch_tap_timeout_ms: u64,
}

impl Default for GestureConfig {
    /// Returns default gesture settings with all standard multi-finger gestures enabled.
    ///
    /// # Returns
    ///
    /// A new [`GestureConfig`] instance with default thresholds.
    fn default() -> Self {
        Self {
            three_finger_swipe_window: true,
            three_finger_swipe_up_maximize: true,
            three_finger_swipe_down_minimize: true,
            four_finger_swipe_window: true,
            four_finger_swipe_down_close: true,
            four_finger_swipe_up_fullscreen: true,
            swipe_threshold: default_swipe_threshold(),
            four_finger_swipe_threshold: default_four_finger_swipe_threshold(),
            touch_two_finger_swipe_window: true,
            touch_two_finger_swipe_down_minimize: true,
            touch_three_finger_tap: true,
            touch_swipe_threshold: default_touch_swipe_threshold(),
            touch_tap_timeout_ms: default_touch_tap_timeout_ms(),
        }
    }
}
