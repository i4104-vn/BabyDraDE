//! Subscriptions, outbox draining, expiry and frame construction helpers.

use std::time::{Duration, Instant};

use common::protocol::{
    BusError, BusErrorCode, Control, Envelope, Kind, MessageId, PeerId, PeerName, Target, Topic,
    REQUEST_TIMEOUT,
};

use super::state::{Offline, Router, State, OFFLINE_LIMIT, OFFLINE_TTL, ROUTER_SOURCE};

impl Router {
    /// Subscribes a peer to a topic and replays the retained value, if any.
    ///
    /// # Arguments
    /// * `id` - Registered connection.
    /// * `topic` - Topic to receive.
    ///
    /// # Errors
    /// Returns [`BusErrorCode::PeerUnavailable`] for an unknown connection or
    /// [`BusErrorCode::InvalidPayload`] before the handshake completes.
    pub fn subscribe(&mut self, id: PeerId, topic: Topic) -> Result<(), BusError> {
        {
            let peer = self.peers.get_mut(&id).ok_or_else(|| {
                BusError::new(BusErrorCode::PeerUnavailable, "unknown connection")
            })?;
            if peer.state != State::Ready {
                return Err(BusError::new(
                    BusErrorCode::InvalidPayload,
                    "peer has not completed the handshake",
                ));
            }
            peer.subscriptions.insert(topic.clone());
        }
        if let Some(retained) = self.retained.get(&topic).cloned() {
            let mut envelope = retained.envelope;
            envelope.retained = true;
            envelope.target = Target::Topic(topic);
            self.push_to(id, envelope);
        }
        Ok(())
    }

    /// Removes one subscription from a peer.
    ///
    /// # Arguments
    /// * `id` - Registered connection.
    /// * `topic` - Topic to stop receiving.
    ///
    /// # Errors
    /// Returns the same errors as [`Self::subscribe`].
    pub fn unsubscribe(&mut self, id: PeerId, topic: &str) -> Result<(), BusError> {
        match self.peers.get_mut(&id) {
            Some(peer) if peer.state == State::Ready => {
                peer.subscriptions.remove(topic);
                Ok(())
            }
            Some(_) => Err(BusError::new(
                BusErrorCode::InvalidPayload,
                "peer has not completed the handshake",
            )),
            None => Err(BusError::new(
                BusErrorCode::PeerUnavailable,
                "unknown connection",
            )),
        }
    }

    /// Takes all frames queued for a connection, leaving the outbox empty.
    pub fn take_outbox(&mut self, id: PeerId) -> Vec<Envelope> {
        self.peers
            .get_mut(&id)
            .map(|peer| std::mem::take(&mut peer.outbox))
            .unwrap_or_default()
    }

    /// Expires pending requests, retained values and offline events.
    ///
    /// # Arguments
    /// * `now` - Current monotonic time supplied by the caller's event loop.
    pub fn tick(&mut self, now: Instant) {
        self.expire_pending(now);
        self.retained.retain(|_, entry| entry.expires_at > now);
        self.expire_offline(now);
    }

    /// Rejects and drops requests older than the response deadline.
    ///
    /// # Arguments
    /// * `now` - Current monotonic time.
    fn expire_pending(&mut self, now: Instant) {
        let expired: Vec<(PeerId, MessageId)> = self
            .pending
            .iter()
            .filter(|(_, pending)| {
                now.saturating_duration_since(pending.started) >= REQUEST_TIMEOUT
            })
            .map(|(key, _)| *key)
            .collect();
        for key in expired {
            if let Some(pending) = self.pending.remove(&key) {
                let error = BusError::new(
                    BusErrorCode::Timeout,
                    format!("request to '{}' timed out", pending.target_name),
                );
                self.reject(pending.origin, pending.origin_name, error, key.1);
            }
        }
    }

    /// Drops expired offline events and their empty queues.
    ///
    /// # Arguments
    /// * `now` - Current monotonic time.
    fn expire_offline(&mut self, now: Instant) {
        for queue in self.offline.values_mut() {
            queue.retain(|entry| entry.expires_at > now);
        }
        self.offline.retain(|_, queue| !queue.is_empty());
    }

    /// Allocates the next correlation id.
    ///
    /// # Returns
    /// A freshly generated message id.
    fn next_id(&mut self) -> MessageId {
        let id = self.next_message_id;
        self.next_message_id = self.next_message_id.wrapping_add(1);
        id
    }

    /// Builds a frame stamped with the router as its source.
    ///
    /// # Arguments
    /// * `target` - Destination selector.
    /// * `kind` - Semantic class of the frame.
    /// * `payload` - JSON body.
    ///
    /// # Returns
    /// A new router-generated envelope.
    pub(super) fn generated(
        &mut self,
        target: Target,
        kind: Kind,
        payload: serde_json::Value,
    ) -> Envelope {
        let mut envelope = Envelope::new(self.next_id(), target, kind, payload);
        envelope.source = ROUTER_SOURCE.to_string();
        envelope
    }

    /// Appends a frame to a peer's outbox when it still exists.
    ///
    /// # Arguments
    /// * `id` - Destination connection.
    /// * `envelope` - Frame to queue.
    pub(super) fn push_to(&mut self, id: PeerId, envelope: Envelope) {
        if let Some(peer) = self.peers.get_mut(&id) {
            peer.outbox.push(envelope);
        }
    }

    /// Queues an error frame answering the origin of a failed frame.
    ///
    /// # Arguments
    /// * `origin` - Connection that receives the error.
    /// * `origin_name` - Name used as the error frame's target.
    /// * `error` - Failure to report.
    /// * `id` - Correlation id of the failed frame.
    pub(super) fn reject(
        &mut self,
        origin: PeerId,
        origin_name: PeerName,
        error: BusError,
        id: MessageId,
    ) {
        let Ok(payload) = serde_json::to_value(&error) else {
            return;
        };
        let mut envelope = self.generated(Target::Peer(origin_name), Kind::Error, payload);
        envelope.id = id;
        self.push_to(origin, envelope);
    }

    /// Publishes a control event to subscribers of a topic.
    ///
    /// # Arguments
    /// * `topic` - Topic to publish on.
    /// * `control` - Control payload to deliver.
    /// * `except` - Connection excluded from delivery, if any.
    pub(super) fn publish_control(
        &mut self,
        topic: Topic,
        control: Control,
        except: Option<PeerId>,
    ) {
        let Ok(payload) = serde_json::to_value(&control) else {
            return;
        };
        let envelope = self.generated(Target::Topic(topic.clone()), Kind::Event, payload);
        let subscribers: Vec<PeerId> = self
            .peers
            .iter()
            .filter(|(id, peer)| {
                peer.state == State::Ready
                    && Some(**id) != except
                    && peer.subscriptions.contains(&topic)
            })
            .map(|(id, _)| *id)
            .collect();
        for id in subscribers {
            self.push_to(id, envelope.clone());
        }
    }

    /// Queues an event for an absent peer, evicting the oldest on overflow.
    ///
    /// # Arguments
    /// * `name` - Name of the absent peer.
    /// * `envelope` - Event frame to hold.
    pub(super) fn queue_offline(&mut self, name: PeerName, envelope: Envelope) {
        let now = Instant::now();
        let ttl = envelope
            .ttl_ms
            .map(Duration::from_millis)
            .unwrap_or(OFFLINE_TTL);
        let expires_at = now.checked_add(ttl).unwrap_or(now);
        let queue = self.offline.entry(name).or_default();
        queue.retain(|entry| entry.expires_at > now);
        if queue.len() >= OFFLINE_LIMIT {
            queue.remove(0);
        }
        queue.push(Offline {
            envelope,
            expires_at,
        });
    }

    /// Delivers unexpired events queued for a peer that just registered.
    ///
    /// # Arguments
    /// * `id` - Newly registered connection.
    /// * `name` - Name whose offline queue is drained.
    pub(super) fn flush_offline(&mut self, id: PeerId, name: &str) {
        let Some(queue) = self.offline.remove(name) else {
            return;
        };
        let now = Instant::now();
        for entry in queue {
            if entry.expires_at > now {
                self.push_to(id, entry.envelope);
            }
        }
    }
}
