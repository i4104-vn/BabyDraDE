//! Z-ordered registry of managed windows.

pub(crate) use crate::modules::window::models::registry::WindowRegistry;

pub(crate) mod cursor_impl;
pub(crate) mod protocol;

pub(crate) use protocol::{cursor, metadata};

#[cfg(test)]
mod tests;
