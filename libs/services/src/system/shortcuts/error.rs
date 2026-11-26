//! Failures returned by shortcut bus operations.

use std::fmt;

/// Failure returned by a shortcut operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutError {
    /// The supplied owner id was empty or whitespace only.
    InvalidOwner,
    /// Connecting to or writing to the bus failed.
    Io(String),
    /// A response body could not be decoded.
    InvalidManifest(String),
    /// The compositor rejected the request or did not answer in time.
    Failed(String),
}

impl fmt::Display for ShortcutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOwner => f.write_str("shortcut owner must not be empty"),
            Self::Io(message) => write!(f, "shortcut bus I/O error: {message}"),
            Self::InvalidManifest(message) => write!(f, "invalid shortcut response: {message}"),
            Self::Failed(message) => write!(f, "shortcut request failed: {message}"),
        }
    }
}

impl std::error::Error for ShortcutError {}
