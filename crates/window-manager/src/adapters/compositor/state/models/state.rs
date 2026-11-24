//! Compositor state data models.

use smithay::reexports::wayland_server::DisplayHandle;

use crate::adapters::window::WindowRegistry;

use crate::adapters::compositor::state::{
    ClipboardState, ConfigSnapshot, InputState, OverlayState, WaylandRuntime,
};

use super::RuntimeResources;

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
    /// Managed window space and runtime resources.
    pub(crate) resources: RuntimeResources,
}
