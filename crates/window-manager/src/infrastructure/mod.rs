//! Frameworks and drivers — the outermost technical layer.
//!
//! `backends` adapts Smithay's Winit/Udev backends, `platform` holds product
//! identifiers, and `process` launches commands. This layer may depend on
//! every inner layer (`adapters`, `application`, `domain`), but nothing inner
//! depends on it.

pub(crate) mod backends;
pub(crate) mod config;
pub(crate) mod platform;
pub(crate) mod process;
pub(crate) mod sensor;
