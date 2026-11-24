use std::{
    ffi::OsString,
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Instant,
};

use smithay::{
    desktop::{PopupManager, Space, Window},
    input::pointer::CursorIcon,
    reexports::{
        calloop::{generic::Generic, EventLoop, Interest, LoopSignal, PostAction},
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            Display, DisplayHandle, Resource,
        },
    },
    utils::{IsAlive, Logical, Point},
    wayland::{
        compositor::CompositorClientState, shell::xdg::ToplevelSurface,
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

pub(crate) struct CalloopData {
    pub(crate) state: BabydraState,
    pub(crate) display_handle: DisplayHandle,
}

pub(crate) struct BabydraState {
    /// Wayland protocol globals and the compositor seat.
    pub(crate) wayland: WaylandRuntime,
    /// Clipboard capture pipeline (MIME queues owned by the component).
    pub(crate) clipboard: ClipboardState,
    /// Cached configuration snapshot owned by the input pipeline.
    pub(crate) config: ConfigSnapshot,
    /// Transient input interaction state.
    pub(crate) input: InputState,
    /// Launcher and Alt+Tab switcher state.
    pub(crate) overlays: OverlayState,
    /// Managed windows in bottom-to-top Z order.
    pub(crate) windows: WindowRegistry,
    /// Shared rendering resources.
    pub(crate) render: RenderState,

    // Flat compatibility fields below are read directly by the composition
    // root and backends, which live outside this adapter crate and cannot be
    // changed as part of this refactor.
    pub(crate) start_time: Instant,
    pub(crate) socket_name: OsString,
    pub(crate) space: Space<Window>,
    pub(crate) loop_signal: LoopSignal,
    pub(crate) popups: PopupManager,

    /// Channel used by the capture thread to hand ownership back to us.
    ///
    /// Compatibility field: the composition root installs this directly, so it
    /// stays on the facade and is forwarded to `ClipboardState` when needed.
    pub(crate) clipboard_sender:
        Option<smithay::reexports::calloop::channel::Sender<Option<String>>>,

    /// Compatibility alias of `render.blur_engine`: the backends (outside this
    /// adapter crate) read this field directly, so both names share one engine.
    pub(crate) blur_engine:
        std::sync::Arc<std::sync::Mutex<crate::adapters::rendering::blur::BlurEngine>>,
    pub(crate) pointer_location: Point<f64, Logical>,
    pub(crate) cursor_visible: bool,
}

/// Compatibility deref for callers outside this adapter crate that still use
/// the historical flat `state.launcher` / `state.alt_tab_switcher` paths. New
/// code should use `state.overlays` explicitly.
impl Deref for BabydraState {
    type Target = OverlayState;

    fn deref(&self) -> &Self::Target {
        &self.overlays
    }
}

impl DerefMut for BabydraState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.overlays
    }
}

mod lifecycle;
mod window_ops;
mod settings;
mod listener;

#[derive(Default)]
pub(crate) struct ClientState {
    pub(crate) compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}

    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}
