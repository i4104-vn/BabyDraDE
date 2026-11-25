//! Versioned message envelope shared by every protocol message.

use serde::{Deserialize, Serialize};

use super::{Event, Request, Response};

/// Protocol version carried by every envelope.
///
/// Bump when a payload changes incompatibly; readers reject mismatched
/// versions instead of mis-decoding a message.
pub const PROTOCOL_VERSION: u16 = 1;

/// Process-local correlation id, also used to match responses.
pub type MessageId = u64;

/// The three kinds of message flowing over the bridge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    /// A feature asking the compositor to do something.
    Request(Request),
    /// The compositor answering a request.
    Response(Response),
    /// A compositor-initiated notification.
    Event(Event),
}

/// One wire message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    /// Protocol version of the sender.
    pub version: u16,
    /// Unique id for this message.
    pub id: MessageId,
    /// Typed message body.
    pub payload: Payload,
}

impl Envelope {
    /// Builds an envelope with the current protocol version.
    pub fn new(id: MessageId, payload: Payload) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            id,
            payload,
        }
    }

    /// Returns `true` when this envelope uses the current version.
    pub fn is_current(&self) -> bool {
        self.version == PROTOCOL_VERSION
    }
}
