//! Touchpad configuration and device setup.

mod handler;
mod models;

pub(crate) use handler::{handle_device_added, handle_device_removed};
