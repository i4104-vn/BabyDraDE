//! Compositor input processing pipeline.

pub(crate) use crate::{
    modules::window::interaction::{
        MoveState, OverlayTouchState, ResizeState, TouchSlotData, TrackpadSwipeBuffer,
    },
    shared::ports::InputContext,
};
pub(crate) use dispatch::{execute_window_intent, process_input_event};
pub(crate) use transform::transform_absolute_position;

pub(crate) mod dispatch;
pub(crate) mod gestures;
pub(crate) mod keyboard;
pub(crate) mod keymap;
pub(crate) mod models;
pub(crate) mod pointer;
pub(crate) mod touch;
pub(crate) mod touchpad;
pub(crate) mod transform;
