//! Input, pointer, and gesture interaction constants.

/// Linux evdev button code for primary left mouse click.
pub const BTN_LEFT: u32 = 272;

/// Linux evdev button code for secondary right mouse click.
pub const BTN_RIGHT: u32 = 273;

/// Linux evdev button code for middle mouse click.
pub const BTN_MIDDLE: u32 = 0x112; // 274

/// Maximum elapsed time between clicks to qualify as a double click (in milliseconds).
pub const DOUBLE_CLICK_TIMEOUT_MS: u128 = 350;

/// Maximum square distance between clicks to qualify as a double click (12px threshold).
pub const DOUBLE_CLICK_DISTANCE_SQ: f64 = 144.0;

/// Movement threshold before committing to interactive drag (4px threshold).
pub const DRAG_THRESHOLD_SQ: f64 = 16.0;

/// Number of fingers for three-finger gestures.
pub const SWIPE_THREE_FINGERS: u32 = 3;

/// Number of fingers for four-finger gestures.
pub const SWIPE_FOUR_FINGERS: u32 = 4;

/// Default pixel displacement required to trigger a swipe gesture.
pub const DEFAULT_SWIPE_THRESHOLD: f64 = 120.0;

/// Delay before a held navigation key starts auto-repeating (milliseconds).
pub const KEY_REPEAT_DELAY_MS: u64 = 260;

/// Interval between auto-repeat steps once the delay has elapsed (milliseconds).
pub const KEY_REPEAT_INTERVAL_MS: u64 = 70;
