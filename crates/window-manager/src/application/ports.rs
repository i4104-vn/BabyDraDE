//! Ports — the boundaries through which use cases reach the outside world.
//!
//! Adapters and infrastructure implement these; the application layer itself
//! only ever sees the plain data and function types declared here.

use crate::domain::shortcuts::Shortcut;

/// Port that launches a configured command (implemented by infrastructure).
pub(crate) type LaunchCommand = fn(String);

/// Port that loads the shortcut table (implemented by the config adapter).
pub(crate) type LoadShortcuts = fn() -> Vec<Shortcut>;

/// Port that loads a fresh gesture policy snapshot.
pub(crate) type LoadGesturePolicy = fn() -> GesturePolicy;

/// Framework-free snapshot of the gesture rules a use case may consult.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GesturePolicy {
    pub(crate) swipe_threshold: f64,
    pub(crate) four_finger_swipe_threshold: f64,
    pub(crate) touch_swipe_threshold: f64,
    pub(crate) touch_tap_timeout_ms: u64,
    pub(crate) three_finger_swipe_window: bool,
    pub(crate) three_finger_swipe_up_maximize: bool,
    pub(crate) three_finger_swipe_down_minimize: bool,
    pub(crate) four_finger_swipe_window: bool,
    pub(crate) four_finger_swipe_up_fullscreen: bool,
    pub(crate) four_finger_swipe_down_close: bool,
    pub(crate) touch_two_finger_swipe_window: bool,
    pub(crate) touch_two_finger_swipe_down_minimize: bool,
    pub(crate) touch_three_finger_tap: bool,
}
