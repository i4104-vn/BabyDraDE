//! Error type for shortcut registration.

use std::fmt;

/// Failure returned while registering or resolving a shortcut.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistrationError {
    /// The combo is malformed or has no modifier.
    InvalidCombo(String),
    /// The command is empty.
    InvalidTarget(String),
    /// The public API was called before the registry was published.
    NotReady,
}

impl fmt::Display for RegistrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCombo(combo) => write!(f, "invalid shortcut combo: {combo:?}"),
            Self::InvalidTarget(message) => write!(f, "invalid shortcut target: {message}"),
            Self::NotReady => f.write_str("shortcut registry is not ready"),
        }
    }
}

impl std::error::Error for RegistrationError {}
