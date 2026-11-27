//! Model types and their impls for the application layer.
//!
//! `BabydraState` and the loop data are in [`runtime`]; the seven compositor
//! components are in [`state`]; port implementations are in [`bridge`].

pub(crate) mod bridge;
pub(crate) mod facade;

// Keep `models::runtime` and `models::state` resolvable for existing callers.
pub(crate) use facade::{runtime, state};

pub(crate) use runtime::{BabydraState, CalloopData, ClientState, RuntimeResources};
pub(crate) use state::{
    ClipboardState, ConfigSnapshot, InputState, OverlayState, RenderState, WaylandRuntime,
    WindowRegistry,
};
