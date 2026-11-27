//! Filesystem services shared by desktop adapters.
//!
//! The `index` module exposes a bounded, one-shot snapshot of the user's home directory
//! so UI adapters can search paths without performing filesystem walks.

mod handler;
mod models;

#[cfg(test)]
mod tests;

pub use handler::scan_home;
pub use models::{FileEntry, MAX_DEPTH, MAX_ENTRIES, SKIP_DIRS};

/// Compatibility alias for the former `index` module.
///
/// Downstream crates import these items through `services::files::index`; the
/// module was flattened into `files`, so this re-export keeps that path valid.
pub mod index {
    pub use super::{scan_home, FileEntry, MAX_DEPTH, MAX_ENTRIES, SKIP_DIRS};
}
