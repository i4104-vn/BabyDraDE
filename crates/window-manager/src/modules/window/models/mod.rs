//! Shared model types owned by the window module.

pub(crate) mod controls;
pub(crate) mod lifecycle;
pub(crate) mod registry;
pub(crate) mod titlebar;

pub(crate) use controls::{interaction, windows};
pub(crate) use lifecycle::{animation, state};
