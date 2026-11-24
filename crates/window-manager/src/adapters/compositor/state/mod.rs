use std::{
    ffi::OsString,
    sync::Arc,
    time::Instant,
};

use smithay::{
    desktop::{PopupManager, Space, Window},
    input::pointer::CursorIcon,
    reexports::{
        calloop::{generic::Generic, EventLoop, Interest, PostAction},
        wayland_server::{
            Display, Resource,
        },
    },
    utils::{IsAlive, Point},
    wayland::{
        shell::xdg::ToplevelSurface,
        socket::ListeningSocketSource,
    },
};

use crate::adapters::window::cursor::resize_cursor_icon;
pub(crate) use crate::adapters::window::interaction::{
    MoveState, OverlayTouchState, ResizeState, TouchGestureBuffer, TrackpadSwipeBuffer,
};
use crate::adapters::window::WindowRegistry;
use crate::application::window_action::WindowAction;

mod clipboard;
mod config;
mod hit_test;
mod input;
mod input_context;
mod overlays;
mod render;
mod wayland;

pub(crate) use clipboard::ClipboardState;
pub(crate) use config::ConfigSnapshot;
pub(crate) use input::InputState;
pub(crate) use overlays::OverlayState;
pub(crate) use render::RenderState;
pub(crate) use wayland::WaylandRuntime;
pub(crate) mod launcher;
mod switcher;
pub(crate) mod titlebar;
pub(crate) mod window;

pub(crate) use window::*;

mod models;

mod lifecycle;
mod window_ops;
mod settings;
mod listener;


pub(crate) use models::{BabydraState, CalloopData, ClientState};
