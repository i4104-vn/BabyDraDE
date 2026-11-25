//! Bounded filesystem index shared with launcher search.
//!
//! Callers request a snapshot once at startup and keep it in memory; no
//! filesystem walk happens on a keystroke.

mod handler;
mod models;

#[cfg(test)]
mod tests;

pub use handler::scan_home;
pub use models::{FileEntry, MAX_DEPTH, MAX_ENTRIES, SKIP_DIRS};
