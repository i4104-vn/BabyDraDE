//! Model types and their impls for the application layer.
//!
//! `BabydraState` and the loop data are in [`runtime`]; the seven compositor
//! components are in [`state`]; port implementations are in [`bridge`].

pub(crate) mod bridge;
pub(crate) mod runtime;
pub(crate) mod state;

pub(crate) use runtime::{BabydraState, CalloopData, ClientState, RuntimeResources};
pub(crate) use state::{
    ClipboardState, ConfigSnapshot, InputState, OverlayState, RenderState, WaylandRuntime,
    WindowRegistry,
};
