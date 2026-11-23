//! Transient input interaction state owned by the compositor.

use std::collections::HashSet;
use std::time::Instant;

use smithay::{
    backend::input::TouchSlot,
    desktop::Window,
    input::pointer::CursorIcon,
    utils::{Logical, Point},
};

use crate::adapters::window::interaction::{
    MoveState, OverlayTouchState, ResizeState, TouchGestureBuffer, TrackpadSwipeBuffer,
};

/// Buffers and flags for move/resize drags, pointer buttons and touch gestures.
#[derive(Default)]
pub(crate) struct InputState {
    pub(crate) move_state: Option<MoveState>,
    pub(crate) resize_state: Option<ResizeState>,
    pub(crate) last_titlebar_click: Option<(Instant, Point<f64, Logical>, Window)>,
    pub(crate) trackpad_swipe: Option<TrackpadSwipeBuffer>,
    /// Last cursor shape requested by the focused Wayland client.
    pub(crate) cursor_icon: CursorIcon,
    pub(crate) pointer_pressed_buttons: HashSet<u32>,
    pub(crate) touch_slots: HashSet<TouchSlot>,
    /// Active single-finger touch interaction with compositor overlays
    /// (launcher or switcher).
    pub(crate) overlay_touch: Option<OverlayTouchState>,
    /// Accumulated per-slot data for multi-touch gesture detection on
    /// touchscreens.
    pub(crate) touch_gesture: TouchGestureBuffer,
}

impl InputState {
    pub(crate) fn new() -> Self {
        Self {
            move_state: None,
            resize_state: None,
            last_titlebar_click: None,
            trackpad_swipe: None,
            cursor_icon: CursorIcon::Default,
            pointer_pressed_buttons: HashSet::new(),
            touch_slots: HashSet::new(),
            overlay_touch: None,
            touch_gesture: TouchGestureBuffer::default(),
        }
    }
}
