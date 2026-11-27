//! Mapping socket and compositor failures into shortcut errors.

use std::io;

use common::protocol::{ProtocolError, Response};

use crate::system::shortcuts::error::ShortcutError;

/// Wraps a socket error as a shortcut failure.
///
/// # Arguments
/// * `error` - Error reported by the bus.
///
/// # Returns
/// A [`ShortcutError::Io`] carrying the error text.
pub(crate) fn io_error(error: io::Error) -> ShortcutError {
    ShortcutError::Io(error.to_string())
}

/// Maps a protocol failure to a shortcut failure.
///
/// # Arguments
/// * `error` - Failure reported by the compositor.
///
/// # Returns
/// A [`ShortcutError::Failed`] preserving the compositor's detail.
pub(super) fn failure(error: &ProtocolError) -> ShortcutError {
    match error {
        ProtocolError::Failed(message) => ShortcutError::Failed(message.clone()),
        other => ShortcutError::Failed(format!("{other:?}")),
    }
}

/// Reports a response that does not fit the issued request.
///
/// # Arguments
/// * `response` - Unexpected response.
///
/// # Returns
/// A [`ShortcutError::Failed`] describing the response.
pub(super) fn unexpected(response: &Response) -> ShortcutError {
    ShortcutError::Failed(format!("unexpected response: {response:?}"))
}
