//! Pointer input subsystem: motions, button clicks, and dragging.


pub(crate) use button::handle_pointer_button;
pub(crate) use motion::{handle_absolute_motion, handle_pointer_motion};

// Inlined from input/pointer/button.rs.
pub(crate) mod button;

// Inlined from input/pointer/config.rs.
pub(crate) mod config;

// Inlined from input/pointer/drag.rs.
pub(crate) mod drag;

// Inlined from input/pointer/motion.rs.
pub(crate) mod motion;
