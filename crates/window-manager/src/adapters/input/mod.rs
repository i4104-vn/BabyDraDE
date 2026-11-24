//! Compositor input processing pipeline.

pub(crate) mod common;
pub(crate) mod keyboard;
pub(crate) mod keymap;
pub(crate) mod pointer;
pub(crate) mod touch;
pub(crate) mod touchpad;

mod handler;

pub(crate) use crate::adapters::window::{
    interaction::{MoveState, OverlayTouchState, ResizeState, TouchSlotData, TrackpadSwipeBuffer},
    InputContext,
};
pub(crate) use handler::{execute_window_intent, process_input_event};
