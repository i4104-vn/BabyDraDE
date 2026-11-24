//! Touch event routing: down, motion, up, cancel and frame.
//!
//! Facade that re-exports the per-family handlers kept in sibling modules.

#[path = "down.rs"]
mod down;
#[path = "lifecycle.rs"]
mod lifecycle;
#[path = "motion.rs"]
mod motion;

pub(crate) use down::handle_touch_down;
pub(crate) use lifecycle::{handle_touch_cancel, handle_touch_frame, handle_touch_up};
pub(crate) use motion::handle_touch_motion;
