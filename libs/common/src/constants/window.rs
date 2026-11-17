//! Window geometry, decoration, and animation constants.

/// Default window corner radius in pixels (floating / normal state).
pub const CORNER_RADIUS: f64 = 14.0;
pub const CORNER_RADIUS_F32: f32 = 14.0;

/// Compositor-rendered server-side decoration titlebar height in logical pixels.
pub const TITLEBAR_HEIGHT: i32 = 34;

/// Titlebar control button hit-test radius in logical pixels.
pub const TITLEBAR_BUTTON_RADIUS: f64 = 12.0;

/// Interactive border resize margin threshold in logical pixels.
pub const BORDER_MARGIN: f64 = 8.0;

/// Minimum allowable window dimensions during interactive resize.
pub const WINDOW_MIN_WIDTH: i32 = 150;
pub const WINDOW_MIN_HEIGHT: i32 = 100;

/// Default duration for window opening transition in milliseconds.
pub const WINDOW_OPEN_ANIM_MS: u64 = 220;

/// Default duration for window closing transition in milliseconds.
pub const WINDOW_CLOSE_ANIM_MS: u64 = 200;
