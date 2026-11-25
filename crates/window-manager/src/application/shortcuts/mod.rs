//! Shortcut dispatch use case.

mod handler;
pub(crate) mod registry;

#[cfg(test)]
mod registry_tests;
#[cfg(test)]
mod tests;

pub(crate) use handler::ShortcutService;
pub use registry::{RegistrationError, ShortcutRegistry};
