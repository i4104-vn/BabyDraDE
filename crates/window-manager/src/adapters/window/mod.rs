//! Shared runtime view of managed windows.
//!
//! This adapter sits below the Wayland, input and rendering adapters so they
//! can exchange window metadata and visual state without importing each other.

pub(crate) mod cursor;
pub(crate) mod input_context;
pub(crate) mod interaction;
pub(crate) mod metadata;
pub(crate) mod registry;

pub(crate) use input_context::InputContext;
pub(crate) use registry::WindowRegistry;
