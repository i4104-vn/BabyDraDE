//! Pointer input subsystem: motions, button clicks, and dragging.

pub(crate) use button::handle_pointer_button;
pub(crate) use gesture::motion::{handle_absolute_motion, handle_pointer_motion};

pub(crate) mod button;
pub(crate) mod config;
pub(crate) mod gesture;
