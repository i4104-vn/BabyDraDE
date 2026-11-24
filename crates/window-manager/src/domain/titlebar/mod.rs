//! Titlebar geometry and hit testing (pure rules).
//!
//! Geometry is expressed with the framework-free value types in
//! [`crate::domain::geometry`]; no window handle, protocol state or I/O is
//! involved.

mod handler;
pub(crate) mod models;

pub(crate) use handler::hit_test;
pub(crate) use models::{FrameInput, TitlebarLayout};

#[cfg(test)]
mod tests;
