//! Wayland protocol state models: runtime globals, client state and selection state.

pub(crate) use clipboard::ClipboardState;
#[allow(unused_imports)]
pub(crate) use client::ClientState;
#[allow(unused_imports)]
pub(crate) use runtime::{ProtocolExtensions, SeatRuntime, WaylandRuntime};

pub(crate) mod clipboard;
pub(crate) mod client;
pub(crate) mod runtime;
