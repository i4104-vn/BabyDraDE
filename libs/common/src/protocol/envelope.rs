//! Wire data model of the in-memory bus.
//!
//! Every frame is one [`Envelope`]. The router stamps `source`, correlates
//! requests with responses, and owns all delivery policy; peers only fill in
//! `target`, `kind` and `payload`.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Version carried by every envelope; readers reject mismatched versions.
pub const PROTOCOL_VERSION: u16 = 2;

/// Correlation id shared by requests, responses and timeouts.
pub type MessageId = u64;
/// Router-local identifier for one connection.
pub type PeerId = u64;
/// Unique name a peer registers with the router.
pub type PeerName = String;
/// Name of a publish/subscribe channel.
pub type Topic = String;

/// Destination of one envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    /// One registered peer, addressed by name.
    Peer(PeerName),
    /// All subscribers of a topic except the sender.
    Topic(Topic),
    /// All ready peers except the sender.
    Broadcast,
    /// The router itself, used for the handshake and subscriptions.
    Router,
}

impl From<&str> for Target {
    /// Converts a borrowed name into a direct peer target.
    ///
    /// # Arguments
    /// * `name` - Name of the target peer.
    ///
    /// # Returns
    /// A [`Target::Peer`] address.
    fn from(name: &str) -> Self {
        Self::Peer(name.to_string())
    }
}

impl From<String> for Target {
    /// Converts an owned name into a direct peer target.
    ///
    /// # Arguments
    /// * `name` - Name of the target peer.
    ///
    /// # Returns
    /// A [`Target::Peer`] address.
    fn from(name: String) -> Self {
        Self::Peer(name)
    }
}

/// Semantic class of one envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// Ask a peer to act; requires a matching [`Kind::Response`].
    Request,
    /// Answer a previously received request.
    Response,
    /// Fire-and-forget notification, optionally retained.
    Event,
    /// Router- or peer-reported failure.
    Error,
}

/// Failure codes reported by the bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusErrorCode {
    /// Another connection already registered this name.
    NameTaken,
    /// The addressed peer does not exist or has disconnected.
    PeerUnavailable,
    /// A request was not answered within its deadline.
    Timeout,
    /// The frame could not be interpreted.
    InvalidPayload,
    /// The frame was not legal for its kind.
    InvalidTarget,
    /// The frame exceeded the codec size limit.
    TooLarge,
    /// The connection was closed.
    Disconnected,
}

impl BusErrorCode {
    /// Returns the stable human-readable label of this code.
    ///
    /// # Returns
    /// The fixed label associated with this failure category.
    fn as_str(self) -> &'static str {
        match self {
            Self::NameTaken => "peer name already taken",
            Self::PeerUnavailable => "peer unavailable",
            Self::Timeout => "request timed out",
            Self::InvalidPayload => "invalid payload",
            Self::InvalidTarget => "invalid target",
            Self::TooLarge => "message too large",
            Self::Disconnected => "disconnected",
        }
    }
}

impl fmt::Display for BusErrorCode {
    /// Writes the stable label of this code into the formatter.
    ///
    /// # Arguments
    /// * `f` - Formatter receiving the label.
    ///
    /// # Errors
    /// Returns the underlying formatter error, if any.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A bus failure with a machine-readable code and a human-readable detail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BusError {
    /// Stable failure category.
    pub code: BusErrorCode,
    /// Context describing which peer, topic or message failed.
    pub message: String,
}

impl BusError {
    /// Builds a failure from a code and a detail message.
    ///
    /// # Arguments
    /// * `code` - Stable failure category.
    /// * `message` - Human-readable context for the failure.
    ///
    /// # Returns
    /// A failure carrying the code and detail message.
    pub fn new(code: BusErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for BusError {
    /// Writes the code and detail message into the formatter.
    ///
    /// # Arguments
    /// * `f` - Formatter receiving the description.
    ///
    /// # Errors
    /// Returns the underlying formatter error, if any.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for BusError {}

/// Control frames exchanged with the router on `Target::Router`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Control {
    /// First frame on a fresh connection; registers `name`.
    Hello {
        /// Requested peer name.
        name: PeerName,
    },
    /// Successful reply to [`Control::Hello`].
    Welcome {
        /// Name accepted by the router.
        name: PeerName,
        /// Names of the peers visible to the newcomer.
        peers: Vec<PeerName>,
    },
    /// Subscribe the sending peer to a topic.
    Subscribe {
        /// Topic to receive.
        topic: Topic,
    },
    /// Stop delivering a topic to the sending peer.
    Unsubscribe {
        /// Topic to stop receiving.
        topic: Topic,
    },
    /// Presence notification published on `bus/peers`.
    PeerJoined {
        /// Name of the newly registered peer.
        name: PeerName,
    },
    /// Presence notification published on `bus/peers`.
    PeerLeft {
        /// Name of the departed peer.
        name: PeerName,
    },
}

/// One wire frame carrying an arbitrary JSON body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    /// Protocol version of the sender.
    pub version: u16,
    /// Correlation id assigned by the sender.
    pub id: MessageId,
    /// Sender name, stamped by the router; empty when sent by a client.
    pub source: PeerName,
    /// Destination selector.
    pub target: Target,
    /// How the frame should be handled.
    pub kind: Kind,
    /// `true` when this frame was replayed from the retained cache.
    pub retained: bool,
    /// Lifetime of a retained or offline frame, in milliseconds.
    pub ttl_ms: Option<u64>,
    /// Typed body serialized as JSON.
    pub payload: serde_json::Value,
}

impl Envelope {
    /// Builds a current-version envelope with no source and no TTL.
    ///
    /// # Arguments
    /// * `id` - Correlation id chosen by the sender.
    /// * `target` - Destination selector.
    /// * `kind` - Semantic class of the frame.
    /// * `payload` - JSON body.
    ///
    /// # Returns
    /// A current-version envelope with no source and no TTL.
    pub fn new(id: MessageId, target: Target, kind: Kind, payload: serde_json::Value) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            id,
            source: String::new(),
            target,
            kind,
            retained: false,
            ttl_ms: None,
            payload,
        }
    }

    /// Returns `true` when this envelope uses the current protocol version.
    ///
    /// # Returns
    /// Whether the frame's version matches [`PROTOCOL_VERSION`].
    pub fn is_current(&self) -> bool {
        self.version == PROTOCOL_VERSION
    }

    /// Decodes the payload as a control frame, when it is one.
    ///
    /// # Returns
    /// The decoded control frame, or `None` when the payload is not one.
    pub fn control(&self) -> Option<Control> {
        serde_json::from_value(self.payload.clone()).ok()
    }

    /// Decodes the payload as a bus failure, when it is one.
    ///
    /// # Returns
    /// The decoded failure, or `None` when the payload is not one.
    pub fn bus_error(&self) -> Option<BusError> {
        serde_json::from_value(self.payload.clone()).ok()
    }
}

/// A delivered envelope with routing details flattened for consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Correlation id of the originating envelope.
    pub id: MessageId,
    /// Registered name of the sender.
    pub source: PeerName,
    /// Semantic class of the message.
    pub kind: Kind,
    /// `true` when this message was replayed from a retained topic.
    pub retained: bool,
    /// Topic through which the message arrived, when applicable.
    pub topic: Option<Topic>,
    /// JSON body of the message.
    pub body: serde_json::Value,
}

impl Message {
    /// Flattens a delivered envelope into a message.
    ///
    /// # Arguments
    /// * `envelope` - Envelope taken from a connection's outbox.
    ///
    /// # Returns
    /// A message with routing details flattened into consumer fields.
    pub fn from_envelope(envelope: Envelope) -> Self {
        let topic = match &envelope.target {
            Target::Topic(topic) => Some(topic.clone()),
            _ => None,
        };
        Self {
            id: envelope.id,
            source: envelope.source,
            kind: envelope.kind,
            retained: envelope.retained,
            topic,
            body: envelope.payload,
        }
    }

    /// Deserializes the body into a concrete type.
    ///
    /// # Arguments
    /// * `T` - Type implementing [`serde::de::DeserializeOwned`].
    ///
    /// # Returns
    /// The decoded body of type `T`.
    ///
    /// # Errors
    /// Returns the JSON error when the body does not match `T`.
    pub fn decode<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.body.clone())
    }
}

/// Anything a client can receive from the bus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Incoming {
    /// A peer asked this client to act.
    Request(Message),
    /// A peer answered one of this client's requests.
    Response(Message),
    /// A peer published an event to a topic or directly to this client.
    Event(Message),
    /// A request exceeded its deadline without an answer.
    Timeout {
        /// Id of the unanswered request.
        id: MessageId,
        /// Name the request was addressed to.
        target: PeerName,
    },
    /// The router or a peer reported a failure.
    Error(BusError),
    /// The connection to the router was closed.
    Disconnected,
}
