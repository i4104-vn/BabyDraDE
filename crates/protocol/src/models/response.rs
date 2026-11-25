//! Responses sent back to feature requests.

use common::ShortcutBinding;
use serde::{Deserialize, Serialize};

use super::{ActiveWindowInfo, MessageId};

/// Failure reason reported by the compositor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolError {
    /// Envelope used an unsupported protocol version.
    UnsupportedVersion,
    /// The compositor has no handler for this request kind.
    UnsupportedRequest,
    /// The requested resource does not exist.
    NotFound,
    /// The compositor could not complete the operation.
    Failed(String),
}

/// Result of one request, correlated by the request's [`MessageId`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Response {
    /// The request was accepted.
    Ok {
        /// Id of the request being answered.
        request_id: MessageId,
    },
    /// Shortcut table snapshot.
    Shortcuts {
        /// Id of the request being answered.
        request_id: MessageId,
        /// Every binding with its owning feature.
        entries: Vec<(String, ShortcutBinding)>,
    },
    /// Active window snapshot.
    ActiveWindow {
        /// Id of the request being answered.
        request_id: MessageId,
        /// Focused window metadata, when any.
        info: Option<ActiveWindowInfo>,
    },
    /// The request failed.
    Error {
        /// Id of the request being answered.
        request_id: MessageId,
        /// Failure reason.
        error: ProtocolError,
    },
}

impl Response {
    /// Returns the id of the request this response answers.
    pub fn request_id(&self) -> MessageId {
        match self {
            Self::Ok { request_id }
            | Self::Shortcuts { request_id, .. }
            | Self::ActiveWindow { request_id, .. }
            | Self::Error { request_id, .. } => *request_id,
        }
    }
}
