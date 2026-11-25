//! Touchscreen input subsystem: touch down, motion, up, and gestures.


pub(crate) use dispatch::{
    handle_touch_cancel, handle_touch_down, handle_touch_frame, handle_touch_motion,
    handle_touch_up,
};

// Inlined from input/touch/dispatch.rs.
pub(crate) mod dispatch;

// Inlined from input/touch/drag.rs.
pub(crate) mod drag;

// Inlined from input/touch/gestures.rs.
pub(crate) mod gestures;
