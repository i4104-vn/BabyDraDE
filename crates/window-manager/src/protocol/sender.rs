//! Compositor-side bus sender backed by a shared framed writer.
//!
//! Unlike the feature-side client, this handle owns no reader thread: the
//! compositor feeds the matching socket into its own event loop. Clones share
//! one writer and one id sequence.

use std::io::{self, ErrorKind};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use serde::Serialize;

use common::protocol::{Control, Envelope, Kind, Message, MessageId, PeerName, Target, Writer};

/// Serializes a message body into the JSON payload of an envelope.
///
/// # Arguments
/// * `body` - Value convertible to JSON.
///
/// # Returns
/// The serialized JSON payload.
///
/// # Errors
/// Returns an [`ErrorKind::InvalidInput`] error when serialization fails.
fn to_value<T: Serialize>(body: &T) -> io::Result<serde_json::Value> {
    serde_json::to_value(body).map_err(|error| io::Error::new(ErrorKind::InvalidInput, error))
}

/// Compositor-side bus handle with no reader thread; safe to clone and call
/// through a shared reference.
#[derive(Clone)]
pub struct BusSender {
    writer: Writer,
    name: PeerName,
}

impl BusSender {
    /// Wraps an already accepted socket in a sender for `name`.
    ///
    /// # Arguments
    /// * `stream` - Socket connected to the router endpoint.
    /// * `name` - Registered name of this local peer, normally `wm`.
    ///
    /// # Returns
    /// A sender sharing a process-unique id sequence with its clones.
    pub fn new(stream: UnixStream, name: PeerName) -> Self {
        Self {
            writer: Writer::new(stream),
            name,
        }
    }

    /// Returns the registered name of this sender.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Builds and writes one envelope, allocating an id when none is given.
    ///
    /// # Arguments
    /// * `target` - Destination selector.
    /// * `kind` - Semantic class of the frame.
    /// * `payload` - Serialized JSON body.
    /// * `id` - Correlation id to reuse, or `None` to allocate one.
    ///
    /// # Returns
    /// The id actually written to the socket.
    ///
    /// # Errors
    /// Returns a socket error when the frame cannot be written.
    fn emit(
        &self,
        target: Target,
        kind: Kind,
        payload: serde_json::Value,
        id: Option<MessageId>,
    ) -> io::Result<MessageId> {
        let id = id.unwrap_or_else(|| self.writer.allocate_id());
        let envelope = Envelope::new(id, target, kind, payload);
        self.writer.write_frame(&envelope)?;
        Ok(id)
    }

    /// Sends a fire-and-forget event to a target.
    ///
    /// # Arguments
    /// * `target` - Peer name, topic, `Broadcast` or `Router`.
    /// * `body` - JSON-serializable event body.
    ///
    /// # Returns
    /// The id assigned to the event.
    ///
    /// # Errors
    /// Returns a serialization or socket error.
    pub fn send<T: Serialize>(&self, target: impl Into<Target>, body: &T) -> io::Result<MessageId> {
        self.emit(target.into(), Kind::Event, to_value(body)?, None)
    }

    /// Sends a plain request and returns its correlation id.
    ///
    /// # Arguments
    /// * `target` - Destination selector.
    /// * `body` - JSON-serializable request body.
    ///
    /// # Returns
    /// The id assigned to the request.
    ///
    /// # Errors
    /// Returns a serialization or socket error.
    pub fn request<T: Serialize>(
        &self,
        target: impl Into<Target>,
        body: &T,
    ) -> io::Result<MessageId> {
        self.emit(target.into(), Kind::Request, to_value(body)?, None)
    }

    /// Answers a request, reusing its id and source.
    ///
    /// # Arguments
    /// * `request` - Request message to answer.
    /// * `body` - JSON-serializable response body.
    ///
    /// # Returns
    /// The id assigned to the response.
    ///
    /// # Errors
    /// Returns a serialization or socket error.
    pub fn respond<T: Serialize>(&self, request: &Message, body: &T) -> io::Result<MessageId> {
        self.emit(
            Target::Peer(request.source.clone()),
            Kind::Response,
            to_value(body)?,
            Some(request.id),
        )
    }

    /// Writes a topic event with the given retention settings.
    ///
    /// # Arguments
    /// * `topic` - Topic to publish on.
    /// * `body` - JSON-serializable event body.
    /// * `retained` - Whether the router keeps the value for late subscribers.
    /// * `ttl_ms` - Optional lifetime of a retained value, in milliseconds.
    ///
    /// # Returns
    /// The id assigned to the event.
    ///
    /// # Errors
    /// Returns a serialization or socket error.
    fn publish_frame<T: Serialize>(
        &self,
        topic: &str,
        body: &T,
        retained: bool,
        ttl_ms: Option<u64>,
    ) -> io::Result<MessageId> {
        let id = self.writer.allocate_id();
        let mut envelope = Envelope::new(
            id,
            Target::Topic(topic.to_string()),
            Kind::Event,
            to_value(body)?,
        );
        envelope.retained = retained;
        envelope.ttl_ms = ttl_ms;
        self.writer.write_frame(&envelope)?;
        Ok(envelope.id)
    }

    /// Publishes an event to a topic.
    ///
    /// # Arguments
    /// * `topic` - Topic to publish on.
    /// * `body` - JSON-serializable event body.
    ///
    /// # Returns
    /// The id assigned to the event.
    ///
    /// # Errors
    /// Returns a serialization or socket error.
    pub fn publish<T: Serialize>(&self, topic: &str, body: &T) -> io::Result<MessageId> {
        self.publish_frame(topic, body, false, None)
    }

    /// Publishes an event that the router retains for late subscribers.
    ///
    /// # Arguments
    /// * `topic` - Topic to publish on.
    /// * `body` - JSON-serializable event body.
    /// * `ttl` - Lifetime of the retained value.
    ///
    /// # Returns
    /// The id assigned to the event.
    ///
    /// # Errors
    /// Returns a serialization or socket error.
    pub fn publish_retained<T: Serialize>(
        &self,
        topic: &str,
        body: &T,
        ttl: Duration,
    ) -> io::Result<MessageId> {
        let ttl_ms = u64::try_from(ttl.as_millis()).unwrap_or(u64::MAX);
        self.publish_frame(topic, body, true, Some(ttl_ms))
    }

    /// Writes a control frame addressed to the router.
    ///
    /// # Arguments
    /// * `control` - Control message to send.
    ///
    /// # Errors
    /// Returns a serialization or socket error.
    fn control(&self, control: &Control) -> io::Result<()> {
        let envelope = Envelope::new(
            self.writer.allocate_id(),
            Target::Router,
            Kind::Request,
            to_value(control)?,
        );
        self.writer.write_frame(&envelope)
    }

    /// Subscribes this handle to a topic.
    ///
    /// # Arguments
    /// * `topic` - Topic to receive.
    ///
    /// # Errors
    /// Returns a serialization or socket error.
    pub fn subscribe(&self, topic: &str) -> io::Result<()> {
        self.control(&Control::Subscribe {
            topic: topic.to_string(),
        })
    }

    /// Stops delivery of a topic to this handle.
    ///
    /// # Arguments
    /// * `topic` - Topic to stop receiving.
    ///
    /// # Errors
    /// Returns a serialization or socket error.
    pub fn unsubscribe(&self, topic: &str) -> io::Result<()> {
        self.control(&Control::Unsubscribe {
            topic: topic.to_string(),
        })
    }

    /// Closes the socket shared by all clones.
    pub fn shutdown(&self) {
        self.writer.shutdown();
    }
}
