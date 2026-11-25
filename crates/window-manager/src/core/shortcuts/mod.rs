//! Shortcut parsing, matching, service and runtime registry.

pub(crate) mod domain;
pub(crate) mod matcher;
pub(crate) mod parser;
pub(crate) mod registry;
mod service;

pub(crate) use registry::ShortcutRegistry;
pub(crate) use service::ShortcutService;

// Compatibility namespace kept so `core::shortcuts::app::*` paths continue to
// resolve for the public registration API and the composition root.
#[allow(clippy::module_inception)]
pub(crate) mod app {
    //! Shortcut dispatch use case.

    pub use crate::core::shortcuts::registry::{RegistrationError, ShortcutRegistry};
    pub(crate) use crate::core::shortcuts::service::ShortcutService;
}
