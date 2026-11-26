//! Wayland protocol state models: runtime globals and selection state.

pub(crate) use clipboard::ClipboardState;
pub(crate) use runtime::WaylandRuntime;

pub(crate) mod clipboard;
pub(crate) mod runtime;
