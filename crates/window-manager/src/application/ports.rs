//! Ports — the boundaries through which use cases reach the outside world.
//!
//! Adapters and infrastructure implement these; the application layer itself
//! only ever sees the plain data and function types declared here.

use std::ffi::OsStr;

use crate::domain::shortcuts::Shortcut;

/// Structured failure returned when a config snapshot cannot be loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConfigError {
    pub(crate) message: String,
}

/// Structured failure returned when a configured command cannot be launched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LaunchError {
    pub(crate) message: String,
}

/// Port that loads the shortcut table (implemented by the config adapter).
///
/// Unlike a bare function pointer, this trait returns a structured error so
/// the application layer can decide how to degrade and tests can fake it.
pub(crate) trait ShortcutRepository {
    fn load(&self) -> Result<Vec<Shortcut>, ConfigError>;
}

/// Port that launches a configured command (implemented by infrastructure).
///
/// Failures are returned to the caller instead of being swallowed, which
/// makes the outcome observable and easy to fake in tests.
pub(crate) trait CommandLauncher {
    fn launch(&self, command: &str) -> Result<(), LaunchError>;
}

/// Legacy port that launches a configured command (implemented by infrastructure).
pub(crate) type LaunchCommand = fn(String);

/// Port that starts one autostart command on the compositor's Wayland display.
///
/// The handle returned on success is kept by the caller so it can be
/// terminated when the compositor exits. The application layer does not need
/// to know what kind of handle the infrastructure uses.
pub(crate) trait AutostartLauncher {
    /// Handle to a successfully started process.
    type Child;

    /// Run `command` through the infrastructure with `WAYLAND_DISPLAY` set to
    /// `wayland_display`.
    fn spawn(&self, command: &str, wayland_display: &OsStr) -> Result<Self::Child, String>;
}

/// Legacy port that loads the shortcut table (implemented by the config adapter).
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
