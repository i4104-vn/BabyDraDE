//! Frame dispatch: validation, correlation and target-specific delivery.

use std::time::{Duration, Instant};

use common::protocol::{
    BusError, BusErrorCode, Control, Envelope, Kind, PeerId, PeerName, Target, Topic,
};

use super::state::{Pending, Retained, Router, State, RETAINED_TTL};

impl Router {
    /// Feeds one decoded frame into the state machine.
    ///
    /// # Arguments
    /// * `from` - Connection that sent the frame.
    /// * `envelope` - Frame to route; its `source` is overwritten.
    pub fn route(&mut self, from: PeerId, mut envelope: Envelope) {
        let Some((state, origin_name)) = self.validate_frame(from, &envelope) else {
            return;
        };
        if state == State::AwaitingHello {
            self.route_hello(from, &origin_name, &envelope);
            return;
        }
        envelope.source = origin_name.clone();
        self.route_ready(from, origin_name, envelope);
    }

    /// Validates the sender connection and the frame's protocol version.
    ///
    /// # Arguments
    /// * `from` - Connection that sent the frame.
    /// * `envelope` - Frame to validate.
    ///
    /// # Returns
    /// The sender's state and name when routing may continue, or `None` after
    /// the frame was silently dropped or answered with an error frame.
    fn validate_frame(&mut self, from: PeerId, envelope: &Envelope) -> Option<(State, PeerName)> {
        let (state, name) = match self.peers.get(&from) {
            Some(peer) if peer.state != State::Closing => {
                (peer.state, peer.name.clone().unwrap_or_default())
            }
            _ => return None,
        };
        if !envelope.is_current() {
            let error = BusError::new(
                BusErrorCode::InvalidPayload,
                format!("unsupported protocol version {}", envelope.version),
            );
            self.reject(from, name, error, envelope.id);
            return None;
        }
        Some((state, name))
    }

    /// Handles the first frame of a connection that is still handshaking.
    ///
    /// # Arguments
    /// * `from` - Handshaking connection.
    /// * `origin_name` - Name currently registered for it, if any.
    /// * `envelope` - First frame received from the connection.
    fn route_hello(&mut self, from: PeerId, origin_name: &str, envelope: &Envelope) {
        let request = envelope.target == Target::Router && envelope.kind == Kind::Request;
        let hello = request.then(|| envelope.control()).flatten();
        if let Some(Control::Hello { name }) = hello {
            let _ = self.hello(from, name);
            return;
        }
        let error = BusError::new(
            BusErrorCode::InvalidTarget,
            "first frame must be a hello request to the router",
        );
        self.reject(from, origin_name.to_string(), error, envelope.id);
        if let Some(peer) = self.peers.get_mut(&from) {
            peer.state = State::Closing;
        }
    }

    /// Routes a frame from a fully registered connection.
    ///
    /// # Arguments
    /// * `from` - Sending connection.
    /// * `origin_name` - Registered name of the sender.
    /// * `envelope` - Frame whose `source` has already been stamped.
    fn route_ready(&mut self, from: PeerId, origin_name: PeerName, envelope: Envelope) {
        if envelope.kind == Kind::Response {
            self.route_response(from, envelope);
            return;
        }
        if envelope.target == Target::Router {
            self.route_control(from, origin_name, envelope);
            return;
        }
        match envelope.kind {
            Kind::Request => self.route_request(from, origin_name, envelope),
            Kind::Event => self.route_event(from, envelope),
            Kind::Response | Kind::Error => {}
        }
    }

    /// Handles request frames addressed to the router.
    ///
    /// # Arguments
    /// * `from` - Sending connection.
    /// * `origin_name` - Registered name of the sender.
    /// * `envelope` - Router control frame.
    fn route_control(&mut self, from: PeerId, origin_name: PeerName, envelope: Envelope) {
        match (envelope.kind, envelope.control()) {
            (Kind::Request, Some(Control::Subscribe { topic })) => {
                let _ = self.subscribe(from, topic);
            }
            (Kind::Request, Some(Control::Unsubscribe { topic })) => {
                let _ = self.unsubscribe(from, &topic);
            }
            _ => {
                let error = BusError::new(
                    BusErrorCode::InvalidPayload,
                    "unsupported router control frame",
                );
                self.reject(from, origin_name, error, envelope.id);
            }
        }
    }

    /// Delivers a request to one ready peer and records it as pending.
    ///
    /// # Arguments
    /// * `from` - Sending connection.
    /// * `origin_name` - Registered name of the sender.
    /// * `envelope` - Request frame.
    fn route_request(&mut self, from: PeerId, origin_name: PeerName, envelope: Envelope) {
        let Some((target_id, target_name)) = self.request_peer(from, &origin_name, &envelope)
        else {
            return;
        };
        self.pending.insert(
            (target_id, envelope.id),
            Pending {
                origin: from,
                origin_name,
                target_name,
                started: Instant::now(),
            },
        );
        self.push_to(target_id, envelope);
    }

    /// Resolves a request's single-peer target, rejecting invalid targets.
    ///
    /// # Arguments
    /// * `from` - Sending connection.
    /// * `origin_name` - Registered name of the sender.
    /// * `envelope` - Request frame whose target is inspected.
    ///
    /// # Returns
    /// The target connection id and name, or `None` after sending an error.
    fn request_peer(
        &mut self,
        from: PeerId,
        origin_name: &str,
        envelope: &Envelope,
    ) -> Option<(PeerId, PeerName)> {
        let Target::Peer(name) = &envelope.target else {
            let error = BusError::new(
                BusErrorCode::InvalidTarget,
                "requests must address one peer",
            );
            self.reject(from, origin_name.to_string(), error, envelope.id);
            return None;
        };
        let name = name.clone();
        let target = self.names.get(&name).copied();
        let ready = target
            .and_then(|id| self.peers.get(&id))
            .is_some_and(|peer| peer.state == State::Ready);
        if let Some(id) = target.filter(|_| ready) {
            return Some((id, name));
        }
        let error = BusError::new(
            BusErrorCode::PeerUnavailable,
            format!("peer '{name}' is not registered"),
        );
        self.reject(from, origin_name.to_string(), error, envelope.id);
        None
    }

    /// Forwards a response to the origin recorded for `(sender, id)`.
    ///
    /// # Arguments
    /// * `from` - Peer sending the response.
    /// * `envelope` - Response frame correlated by sender and id.
    fn route_response(&mut self, from: PeerId, envelope: Envelope) {
        if let Some(pending) = self.pending.remove(&(from, envelope.id)) {
            self.push_to(pending.origin, envelope);
        }
    }

    /// Delivers an event according to its target selector.
    ///
    /// # Arguments
    /// * `from` - Sending connection.
    /// * `envelope` - Event frame.
    fn route_event(&mut self, from: PeerId, envelope: Envelope) {
        match envelope.target.clone() {
            Target::Peer(name) => self.route_peer_event(name, envelope),
            Target::Topic(topic) => self.route_topic_event(from, topic, envelope),
            Target::Broadcast => self.route_broadcast_event(from, envelope),
            // `route_ready` diverts every Router-targeted frame to `route_control`
            // before dispatching by kind, so this arm is unreachable.
            Target::Router => {}
        }
    }

    /// Delivers an event to one peer, or queues it while the peer is offline.
    ///
    /// # Arguments
    /// * `name` - Name of the target peer.
    /// * `envelope` - Event frame.
    fn route_peer_event(&mut self, name: PeerName, envelope: Envelope) {
        let target = self.names.get(&name).copied();
        let ready = target
            .and_then(|id| self.peers.get(&id))
            .is_some_and(|peer| peer.state == State::Ready);
        if let Some(id) = target.filter(|_| ready) {
            self.push_to(id, envelope);
        } else {
            self.queue_offline(name, envelope);
        }
    }

    /// Fans an event out to subscribers and caches it when retained.
    ///
    /// # Arguments
    /// * `from` - Sending connection, excluded from delivery.
    /// * `topic` - Topic to publish on.
    /// * `envelope` - Event frame.
    fn route_topic_event(&mut self, from: PeerId, topic: Topic, envelope: Envelope) {
        let subscribers: Vec<PeerId> = self
            .peers
            .iter()
            .filter(|(id, peer)| {
                peer.state == State::Ready && **id != from && peer.subscriptions.contains(&topic)
            })
            .map(|(id, _)| *id)
            .collect();
        for id in subscribers {
            self.push_to(id, envelope.clone());
        }
        if envelope.retained {
            self.retain_event(topic, envelope);
        }
    }

    /// Stores a retained event with its effective expiration time.
    ///
    /// # Arguments
    /// * `topic` - Topic whose retained value is replaced.
    /// * `envelope` - Event frame to cache.
    fn retain_event(&mut self, topic: Topic, envelope: Envelope) {
        let now = Instant::now();
        let ttl = envelope
            .ttl_ms
            .map(Duration::from_millis)
            .unwrap_or(RETAINED_TTL);
        self.retained.insert(
            topic,
            Retained {
                envelope,
                expires_at: now.checked_add(ttl).unwrap_or(now),
            },
        );
    }

    /// Fans an event out to every ready peer except the sender.
    ///
    /// # Arguments
    /// * `from` - Sending connection, excluded from delivery.
    /// * `envelope` - Event frame.
    fn route_broadcast_event(&mut self, from: PeerId, envelope: Envelope) {
        let recipients: Vec<PeerId> = self
            .peers
            .iter()
            .filter(|(id, peer)| peer.state == State::Ready && **id != from)
            .map(|(id, _)| *id)
            .collect();
        for id in recipients {
            self.push_to(id, envelope.clone());
        }
    }
}
