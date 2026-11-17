//! Centralized design tokens, geometry, and runtime constants for BabyDraDE.

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

/// Mouse button codes (evdev / linux input event codes).
pub const BTN_LEFT: u32 = 272;
pub const BTN_RIGHT: u32 = 273;
pub const BTN_MIDDLE: u32 = 0x112; // 274

/// Threshold for double click / double tap detection.
pub const DOUBLE_CLICK_TIMEOUT_MS: u128 = 350;
pub const DOUBLE_CLICK_DISTANCE_SQ: f64 = 144.0; // 12.0px

/// Movement threshold before committing to interactive drag.
pub const DRAG_THRESHOLD_SQ: f64 = 16.0; // 4.0px

/// Maximum visible previews in the switcher dock.
pub const DOCK_MAX_VISIBLE_ITEMS: usize = 5;
