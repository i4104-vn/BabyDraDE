//! Shared runtime view of managed windows.
//!
//! This adapter sits below the Wayland, input and rendering adapters so they
//! can exchange window metadata and visual state without importing each other.

pub(crate) mod cursor;
pub(crate) mod metadata;
