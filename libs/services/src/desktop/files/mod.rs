//! Filesystem services shared by desktop adapters.
//!
//! The `index` module exposes a bounded, one-shot snapshot of the user's home directory
//! so UI adapters can search paths without performing filesystem walks.

pub mod index;

pub use index::{scan_home, FileEntry, MAX_DEPTH, MAX_ENTRIES, SKIP_DIRS};
