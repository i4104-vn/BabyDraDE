//! File-based shortcut registration bridge.
//!
//! Watches manifests published by feature apps and applies them to the
//! application-level registry consumed by the keyboard pipeline.

mod handler;

#[cfg(test)]
mod tests;

pub(crate) use handler::spawn_watcher;
