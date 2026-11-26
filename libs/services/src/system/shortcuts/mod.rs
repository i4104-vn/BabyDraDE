//! Bus-backed shortcut registration declarations and re-exports.

mod client;
mod error;
mod snapshot;
mod transport;

#[cfg(test)]
mod tests;

pub use client::ShortcutClient;
pub use common::{ShortcutBinding, ShortcutManifest, ShortcutTarget};
pub use error::ShortcutError;
pub use snapshot::read_all;
