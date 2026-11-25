//! Input interaction state shared between the input pipeline and the compositor.
//!
//! These types describe in-progress pointer/touch/gesture interactions. They
//! live in the neutral window adapter so neither side owns them exclusively.

pub(crate) use crate::modules::window::models::interaction::{
    MoveState, OverlayTouchState, ResizeState, TouchGestureBuffer, TouchSlotData,
    TrackpadSwipeBuffer,
};
