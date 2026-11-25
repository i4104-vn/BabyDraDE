//! Touchpad input subsystem: scroll, swipe, gestures, and device configuration.


pub(crate) use config::{handle_device_added, handle_device_removed};
pub(crate) use gestures::{
    handle_hold_begin, handle_hold_end, handle_pinch_begin, handle_pinch_end, handle_pinch_update,
};
pub(crate) use scroll::handle_axis;
pub(crate) use swipe::{handle_swipe_begin, handle_swipe_end, handle_swipe_update};

// Inlined from input/touchpad/config.rs.
pub(crate) mod config;

// Inlined from input/touchpad/gestures.rs.
pub(crate) mod gestures;

// Inlined from input/touchpad/scroll.rs.
pub(crate) mod scroll;

// Inlined from input/touchpad/swipe.rs.
pub(crate) mod swipe;
