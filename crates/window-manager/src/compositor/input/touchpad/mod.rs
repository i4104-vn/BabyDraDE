//! Touchpad input subsystem: scroll, swipe, gestures, and device configuration.

pub mod config;
pub mod gestures;
pub mod scroll;
pub mod swipe;

#[allow(unused_imports)]
pub use config::TouchpadConfig;
pub(crate) use config::{handle_device_added, handle_device_removed};
pub(crate) use gestures::{
    handle_hold_begin, handle_hold_end, handle_pinch_begin, handle_pinch_end, handle_pinch_update,
};
pub(crate) use scroll::handle_axis;
pub(crate) use swipe::{handle_swipe_begin, handle_swipe_end, handle_swipe_update};
