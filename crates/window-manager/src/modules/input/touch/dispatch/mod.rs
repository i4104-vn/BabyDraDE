//! Multi-touch input event handlers: touch down, motion, up, cancel, and frame.

pub(crate) mod gesture_state;
pub(crate) mod overlay;

mod handlers;

pub(crate) use handlers::{
    handle_touch_cancel, handle_touch_down, handle_touch_frame, handle_touch_motion,
    handle_touch_up,
};

#[cfg(test)]
mod tests;
