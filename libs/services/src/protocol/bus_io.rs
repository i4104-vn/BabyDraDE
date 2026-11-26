//! Public request, publish and receive API of the feature bus.

use std::io;
use std::sync::PoisonError;
use std::time::{Duration, Instant};

use serde::Serialize;

use common::protocol::writer::{lock, to_value};
use common::protocol::{Control, Envelope, Incoming, Kind, Message, MessageId, PeerName, Target};

use super::bus::Bus;
use super::state::{ReaderPending, REQUEST_TIMEOUT};

impl Bus {
    /// Sends a fire-and-forget event to a target.
    ///
    /// # Arguments
    /// * `target` - Peer, topic, `Broadcast` or `Router`.
    /// * `body` - JSON-serializable event body.
    ///
    /// # Returns
    /// The id assigned to the event.
    ///
    /// # Errors
    /// Returns a serialization or socket error.
    pub fn send<T: Serialize>(&self, target: impl Into<Target>, body: &T) -> io::Result<MessageId> {
        let id = self.writer.allocate_id();
        self.emit(id, target.into(), Kind::Event, to_value(body)?)
    }

    /// Sends a request and records its local deadline.
    ///
    /// # Arguments
    /// * `target` - Usually a peer name; the router rejects other targets.
    /// * `body` - JSON-serializable request body.
    ///
    /// # Returns
    /// The id used to correlate the response.
    ///
    /// # Errors
    /// Returns a serialization or socket error; the pending entry is removed
    /// when writing fails.
    pub fn request<T: Serialize>(
        &self,
        target: impl Into<Target>,
        body: &T,
    ) -> io::Result<MessageId> {
        let id = self.writer.allocate_id();
        let target = target.into();
        let name = match &target {
            Target::Peer(name) => name.clone(),
            Target::Topic(topic) => topic.clone(),
            Target::Broadcast => "broadcast".to_string(),
            Target::Router => "router".to_string(),
        };
        let payload = to_value(body)?;
        let deadline = Instant::now()
            .checked_add(REQUEST_TIMEOUT)
            .unwrap_or_else(Instant::now);
        lock(&self.state.pending).insert(
            id,
            ReaderPending {
                deadline,
                target: name,
            },
        );
        if let Err(error) = self.emit(id, target, Kind::Request, payload) {
            lock(&self.state.pending).remove(&id);
            return Err(error);
        }
        Ok(id)
    }

    /// Answers a received request, reusing its id and source.
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
            request.id,
            Target::Peer(request.source.clone()),
            Kind::Response,
            to_value(body)?,
        )
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

    /// Publishes an event the router retains for late subscribers.
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

    /// Writes one topic event with optional retention.
    ///
    /// # Arguments
    /// * `topic` - Topic to publish on.
    /// * `body` - JSON-serializable event body.
    /// * `retained` - Whether the router keeps the value.
    /// * `ttl_ms` - Optional lifetime in milliseconds.
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
        let mut envelope = Envelope::new(
            self.writer.allocate_id(),
            Target::Topic(topic.to_string()),
            Kind::Event,
            to_value(body)?,
        );
        envelope.retained = retained;
        envelope.ttl_ms = ttl_ms;
        self.writer.write_frame(&envelope)?;
        Ok(envelope.id)
    }

    /// Writes a control frame addressed to the router.
    ///
    /// # Arguments
    /// * `control` - Control message to send.
    ///
    /// # Errors
    /// Returns a serialization or socket error.
    fn control(&self, control: &Control) -> io::Result<()> {
        self.emit(
            self.writer.allocate_id(),
            Target::Router,
            Kind::Request,
            to_value(control)?,
        )?;
        Ok(())
    }

    /// Subscribes to a topic; retained values are replayed by the router.
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

    /// Stops delivery of a topic to this client.
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

    /// Returns one queued message, if available, after pruning timeouts.
    pub fn try_recv(&self) -> Option<Incoming> {
        let mut messages = lock(&self.state.inbox.messages);
        self.prune(&mut messages);
        messages.pop_front()
    }

    /// Waits at most `timeout` for one queued message.
    ///
    /// # Arguments
    /// * `timeout` - Maximum time to wait.
    ///
    /// # Returns
    /// The next incoming message, or `None` on timeout.
    pub fn recv_timeout(&self, timeout: Duration) -> Option<Incoming> {
        let deadline = Instant::now()
            .checked_add(timeout)
            .unwrap_or_else(Instant::now);
        let mut messages = lock(&self.state.inbox.messages);
        loop {
            self.prune(&mut messages);
            if let Some(incoming) = messages.pop_front() {
                return Some(incoming);
            }
            let now = Instant::now();
            if now >= deadline {
                self.prune(&mut messages);
                return messages.pop_front();
            }
            let (guard, _) = self
                .state
                .inbox
                .wake
                .wait_timeout(messages, deadline.saturating_duration_since(now))
                .unwrap_or_else(PoisonError::into_inner);
            messages = guard;
        }
    }

    /// Drains all queued messages after pruning timed-out requests.
    pub fn drain(&self) -> Vec<Incoming> {
        let mut messages = lock(&self.state.inbox.messages);
        self.prune(&mut messages);
        messages.drain(..).collect()
    }

    /// Returns the sorted peer list, including this client.
    pub fn peers(&self) -> Vec<PeerName> {
        self.state
            .peers
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}
