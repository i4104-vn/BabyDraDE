//! Shortcut entities and rules (pure domain logic).
//!
//! This module knows nothing about config files, processes or the event loop.
//! Keysym identifiers come from the framework-free [`super::keysyms`] table.

mod handler;
pub(crate) mod models;

pub(crate) use handler::{command_for, parse_combo, parse_modifiers, to_keysym};
pub(crate) use models::{Mods, Shortcut};

#[cfg(test)]
mod tests;
