//! Pointer input subsystem: motions, button clicks, and dragging.

pub(crate) mod button;
pub(crate) mod config;
pub(crate) mod drag;
pub(crate) mod motion;

pub(crate) use button::handle_pointer_button;
pub(crate) use motion::{handle_pointer_motion, handle_absolute_motion};
