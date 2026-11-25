//! Errors produced while encoding or decoding envelopes.

use std::fmt;

/// Protocol codec failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    /// JSON serialization or deserialization failed.
    InvalidJson(String),
    /// The transport could not deliver the message.
    Io(String),
    /// Envelope carried a version this process does not understand.
    UnsupportedVersion {
        /// Version found on the wire.
        found: u16,
        /// Version expected by this process.
        expected: u16,
    },
    /// Encoded message exceeded the safety limit.
    TooLarge {
        /// Actual size in bytes.
        size: usize,
        /// Allowed size in bytes.
        limit: usize,
    },
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(message) => write!(f, "invalid protocol JSON: {message}"),
            Self::Io(message) => write!(f, "protocol transport error: {message}"),
            Self::UnsupportedVersion { found, expected } => {
                write!(f, "protocol version {found} (expected {expected})")
            }
            Self::TooLarge { size, limit } => {
                write!(f, "protocol message too large: {size} bytes (limit {limit})")
            }
        }
    }
}

impl std::error::Error for CodecError {}
