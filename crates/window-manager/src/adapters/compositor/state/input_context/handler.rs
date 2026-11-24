//! `InputContext` trait implementation for the compositor state.

use std::{collections::HashSet, time::Instant};

use common::Action;
use smithay::utils::{Logical, Point, Serial, Size};
use smithay::wayland::shell::wlr_layer::Layer;
use smithay::{
    backend::input::{KeyState, TouchSlot},
    desktop::Space,
    desktop::Window,
    input::{
        keyboard::{FilterResult, Keycode, KeysymHandle, ModifiersState},
        pointer::{
            AxisFrame, ButtonEvent, GestureHoldBeginEvent, GestureHoldEndEvent,
            GesturePinchBeginEvent, GesturePinchEndEvent, GesturePinchUpdateEvent,
            GestureSwipeBeginEvent, GestureSwipeEndEvent, GestureSwipeUpdateEvent,
            MotionEvent as PointerMotionEvent,
        },
        touch::{
            DownEvent as TouchDownEvent, MotionEvent as TouchMotionEvent, UpEvent as TouchUpEvent,
        },
    },
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
        wayland_server::protocol::wl_surface::WlSurface,
    },
};

use crate::adapters::desktop::launcher::search::SearchText;
use crate::adapters::window::InputContext;
use crate::application::ports::GesturePolicy;
use crate::domain::shortcuts::Mods;

use super::super::{
    window::state_data::lock_window_state, BabydraState, MoveState, OverlayTouchState, ResizeState,
    TouchGestureBuffer, TrackpadSwipeBuffer,
};

impl InputContext for BabydraState {
    windows_methods!();
    config_methods!();
    launcher_methods!();
    switcher_methods!();
    devices_methods!();
}
