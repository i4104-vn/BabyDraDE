//! Pointer input subsystem: motions, button clicks, and dragging.

pub mod button;
pub mod config;
pub mod drag;
pub mod motion;

pub(crate) use button::handle_pointer_button;
pub(crate) use config::configure_mouse;
pub use config::MouseConfig;
pub(crate) use motion::{handle_pointer_motion, handle_pointer_motion_absolute};
