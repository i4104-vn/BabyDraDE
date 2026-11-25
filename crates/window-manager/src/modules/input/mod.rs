//! Compositor input processing pipeline.

pub(crate) use crate::{
    shared::ports::InputContext,
    modules::window::interaction::{
        MoveState, OverlayTouchState, ResizeState, TouchSlotData, TrackpadSwipeBuffer,
    },
};
pub(crate) use dispatch::{execute_window_intent, process_input_event};

/// Compatibility path for the former `input::common` module.
#[allow(unused_imports)]
pub(crate) mod common;

pub(crate) mod models;
pub(crate) mod keyboard;
pub(crate) mod pointer;
pub(crate) mod touch;
pub(crate) mod touchpad;
pub(crate) mod transform;
pub(crate) mod dispatch;
pub(crate) mod gestures;
pub(crate) mod keymap;
