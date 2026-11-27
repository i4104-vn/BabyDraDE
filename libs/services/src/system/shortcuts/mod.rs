//! Bus-backed shortcut registration declarations and re-exports.

mod api;
mod transport;

#[cfg(test)]
mod tests;

pub(crate) use api::error;
#[cfg(test)]
pub(crate) use api::snapshot;

pub use api::client::ShortcutClient;
pub use api::error::ShortcutError;
pub use api::snapshot::read_all;
pub use common::{ShortcutBinding, ShortcutManifest, ShortcutTarget};
