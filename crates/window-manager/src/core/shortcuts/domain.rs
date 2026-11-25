//! Shortcut entities and rules (pure domain logic).
//!
//! This module knows nothing about config files, processes or the event loop.
//! Keysym identifiers come from the framework-free keysyms table.
//! The rule models themselves live in `crate::core::models`; this module keeps
//! the parser/matcher paths resolvable for existing callers.

pub(crate) use crate::core::models::shortcuts::{Mods, ResolvedShortcut, Shortcut};

pub(crate) use super::matcher::command_for;
pub(crate) use super::parser::{parse_combo, parse_modifiers, to_keysym};
