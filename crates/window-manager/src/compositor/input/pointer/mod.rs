//! Pointer input subsystem: motions, button clicks, and dragging.

pub mod button;
pub mod drag;
pub mod motion;

pub(crate) use button::handle_pointer_button;
pub(crate) use motion::{handle_pointer_motion, handle_pointer_motion_absolute};

