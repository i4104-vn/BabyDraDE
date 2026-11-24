//! Compositor state data models.

use std::{
    ffi::OsString,
    ops::{Deref, DerefMut},
    time::Instant,
};

use smithay::{
    desktop::{PopupManager, Space, Window},
    reexports::{calloop::LoopSignal, wayland_server::DisplayHandle},
    utils::{Logical, Point},
};

use crate::adapters::window::WindowRegistry;

use crate::adapters::compositor::state::{
    ClipboardState, ConfigSnapshot, InputState, OverlayState, RenderState, WaylandRuntime,
};

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

