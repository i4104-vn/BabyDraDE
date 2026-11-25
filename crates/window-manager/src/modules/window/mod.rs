//! Neutral window data, lifecycle and hit testing.
//!
//! This module sits below input, rendering and shell so they exchange window
//! state without importing one another.

pub(crate) mod animation;
pub(crate) mod models;
pub(crate) mod hit_test;
pub(crate) mod interaction;
pub(crate) mod ops;
pub(crate) mod registry;
pub(crate) mod sizing;
pub(crate) mod state_data;
pub(crate) mod titlebar;
pub(crate) mod windows;

pub(crate) use registry::{cursor, cursor_impl, metadata, WindowRegistry};
