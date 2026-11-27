//! Connection lifecycle: allocation, handshake, teardown and peer listing.

use std::collections::{HashMap, HashSet};

use common::protocol::{
    BusError, BusErrorCode, Control, Envelope, Kind, MessageId, PeerId, PeerName, Target, Topic,
};

use super::state::{Peer, Router, State, PEERS_TOPIC};

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    /// Creates an empty router.
    pub fn new() -> Self {
        Self {
            next_peer_id: 1,
            next_message_id: 1,
            peers: HashMap::new(),
            names: HashMap::new(),
            pending: HashMap::new(),
            retained: HashMap::new(),
            offline: HashMap::new(),
        }
    }

    /// Allocates a connection that must start with a `Hello` frame.
    pub fn add_connection(&mut self) -> PeerId {
        self.insert_peer(None)
    }

    /// Pre-registers a peer that skips the handshake, such as the WM socket.
    ///
    /// # Arguments
    /// * `name` - Name to reserve for the local peer.
    pub fn register_local(&mut self, name: PeerName) -> PeerId {
        let id = self.insert_peer(Some(name.clone()));
        self.names.insert(name, id);
        id
    }

    /// Allocates a connection, starting it ready when a name is supplied.
    ///
    /// # Arguments
    /// * `name` - Name to pre-register, or `None` to await a handshake.
    ///
    /// # Returns
    /// The id assigned to the new connection.
    fn insert_peer(&mut self, name: Option<PeerName>) -> PeerId {
        let id = self.next_peer_id;
        self.next_peer_id = self.next_peer_id.wrapping_add(1);
        let state = if name.is_some() {
            State::Ready
        } else {
            State::AwaitingHello
        };
        self.peers.insert(
            id,
            Peer {
                name,
                state,
                subscriptions: HashSet::new(),
                outbox: Vec::new(),
            },
        );
        id
    }

    /// Completes the handshake of an awaiting connection.
    ///
    /// On success a `Welcome` frame is queued and `PeerJoined` is published on
    /// `bus/peers`. On a duplicate name an `Error` frame is queued and the
    /// connection is marked for removal.
    ///
    /// # Arguments
    /// * `id` - Connection allocated by [`Self::add_connection`].
    /// * `name` - Requested peer name.
    ///
    /// # Errors
    /// Returns [`BusErrorCode::NameTaken`], [`BusErrorCode::PeerUnavailable`]
    /// for an unknown connection, or [`BusErrorCode::InvalidPayload`] when the
    /// connection already completed its handshake.
    pub fn hello(&mut self, id: PeerId, name: PeerName) -> Result<(), BusError> {
        self.validate_handshake(id)?;
        self.check_name(id, &name)?;
        let welcome = self.welcome_frame(&name)?;
        if let Some(peer) = self.peers.get_mut(&id) {
            peer.name = Some(name.clone());
            peer.state = State::Ready;
            peer.outbox.push(welcome);
        }
        self.names.insert(name.clone(), id);
        self.flush_offline(id, &name);
        self.publish_control(
            Topic::from(PEERS_TOPIC),
            Control::PeerJoined { name },
            Some(id),
        );
        Ok(())
    }

    /// Checks that a connection is still awaiting its handshake.
    ///
    /// # Arguments
    /// * `id` - Connection to validate.
    ///
    /// # Errors
    /// Returns [`BusErrorCode::InvalidPayload`] after a completed handshake, or
    /// [`BusErrorCode::PeerUnavailable`] for an unknown or closing connection.
    fn validate_handshake(&self, id: PeerId) -> Result<(), BusError> {
        match self.peers.get(&id).map(|peer| peer.state) {
            Some(State::AwaitingHello) => Ok(()),
            Some(State::Ready) => Err(BusError::new(
                BusErrorCode::InvalidPayload,
                "peer already completed the handshake",
            )),
            _ => Err(BusError::new(
                BusErrorCode::PeerUnavailable,
                "unknown or closing connection",
            )),
        }
    }

    /// Rejects a name already owned by another connection.
    ///
    /// # Arguments
    /// * `id` - Connection requesting the name.
    /// * `name` - Requested peer name.
    ///
    /// # Errors
    /// Returns [`BusErrorCode::NameTaken`] and marks the connection as closing
    /// when another connection already owns the name.
    fn check_name(&mut self, id: PeerId, name: &PeerName) -> Result<(), BusError> {
        let Some(owner) = self.names.get(name).copied() else {
            return Ok(());
        };
        let error = BusError::new(
            BusErrorCode::NameTaken,
            format!("peer name '{name}' is already used by connection {owner}"),
        );
        self.reject(id, name.clone(), error.clone(), 0);
        if let Some(peer) = self.peers.get_mut(&id) {
            peer.state = State::Closing;
        }
        Err(error)
    }

    /// Builds the `Welcome` reply listing all registered peers.
    ///
    /// # Arguments
    /// * `name` - Name accepted for the new peer.
    ///
    /// # Returns
    /// A router-generated response envelope.
    ///
    /// # Errors
    /// Returns [`BusErrorCode::InvalidPayload`] when serialization fails.
    fn welcome_frame(&mut self, name: &PeerName) -> Result<Envelope, BusError> {
        let payload = serde_json::to_value(Control::Welcome {
            name: name.clone(),
            peers: self.peers_list(),
        })
        .map_err(|error| BusError::new(BusErrorCode::InvalidPayload, error.to_string()))?;
        Ok(self.generated(Target::Router, Kind::Response, payload))
    }

    /// Removes a connection and notifies peers waiting on it.
    ///
    /// # Arguments
    /// * `id` - Connection to remove.
    pub fn remove(&mut self, id: PeerId) {
        let Some(peer) = self.peers.remove(&id) else {
            return;
        };
        self.release_name(id, peer.name.as_ref());
        self.fail_pending(id);
        if let Some(name) = peer.name {
            self.publish_control(
                Topic::from(PEERS_TOPIC),
                Control::PeerLeft { name },
                Some(id),
            );
        }
    }

    /// Unregisters a name when the departing connection still owns it.
    ///
    /// # Arguments
    /// * `id` - Departing connection.
    /// * `name` - Name it registered, if any.
    fn release_name(&mut self, id: PeerId, name: Option<&PeerName>) {
        let Some(name) = name else {
            return;
        };
        if self.names.get(name) == Some(&id) {
            self.names.remove(name);
        }
    }

    /// Rejects pending requests involving a departing connection.
    ///
    /// # Arguments
    /// * `id` - Departing connection.
    fn fail_pending(&mut self, id: PeerId) {
        let keys: Vec<(PeerId, MessageId)> = self
            .pending
            .keys()
            .filter(|(target, origin)| *target == id || *origin == id)
            .copied()
            .collect();
        for key in keys {
            if let Some(pending) = self.pending.remove(&key) {
                if pending.origin != id {
                    let error = BusError::new(
                        BusErrorCode::PeerUnavailable,
                        format!("peer '{}' disconnected", pending.target_name),
                    );
                    self.reject(pending.origin, pending.origin_name, error, key.1);
                }
            }
        }
    }

    /// Returns the names of all registered peers in stable order.
    pub fn peers(&self) -> Vec<PeerName> {
        self.peers_list()
    }

    /// Returns the names of all registered peers in stable order.
    ///
    /// # Returns
    /// Sorted names of every peer currently owning a name.
    fn peers_list(&self) -> Vec<PeerName> {
        let mut names: Vec<PeerName> = self.names.keys().cloned().collect();
        names.sort();
        names
    }
}
