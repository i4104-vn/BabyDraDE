//! Connection lifecycle for the feature-side bus.

use std::collections::HashMap;
use std::io::{self, ErrorKind, Write};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use common::protocol::writer::{lock, to_value, Writer, WRITE_TIMEOUT};
use common::protocol::{
    connect, encode, Control, Envelope, Incoming, Kind, MessageId, PeerName, Target,
};

use super::state::{await_welcome, reader_loop, with_self, BusState, Inbox, HANDSHAKE_TIMEOUT};

/// Full feature-side bus: one writer and a background reader thread.
pub struct Bus {
    pub(crate) name: PeerName,
    pub(crate) writer: Writer,
    pub(crate) state: Arc<BusState>,
    pub(crate) reader: Mutex<Option<JoinHandle<()>>>,
}

impl Bus {
    /// Connects to the router, completes the handshake and starts the reader.
    ///
    /// # Arguments
    /// * `name` - Unique peer name to register with the router.
    ///
    /// # Returns
    /// A ready bus client.
    ///
    /// # Errors
    /// Returns a socket error when no router is listening, the handshake
    /// times out, or the router rejects the name.
    pub fn connect(name: &str) -> io::Result<Self> {
        let mut stream = connect()?;
        stream.set_read_timeout(Some(HANDSHAKE_TIMEOUT))?;
        stream.set_write_timeout(Some(WRITE_TIMEOUT))?;
        let hello = Envelope::new(
            0,
            Target::Router,
            Kind::Request,
            to_value(&Control::Hello {
                name: name.to_string(),
            })?,
        );
        let line = encode(&hello).map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
        stream.write_all(&line)?;
        let (peers, early) = await_welcome(&mut stream)?;
        stream.set_read_timeout(None)?;
        let reader_stream = stream.try_clone()?;
        let state = Arc::new(BusState {
            name: name.to_string(),
            inbox: Inbox {
                messages: Mutex::new(early),
                wake: Condvar::new(),
            },
            peers: RwLock::new(with_self(name, peers)),
            pending: Mutex::new(HashMap::new()),
            stop: AtomicBool::new(false),
        });
        let reader_state = Arc::clone(&state);
        let reader = thread::Builder::new()
            .name(format!("bus-reader-{name}"))
            .spawn(move || reader_loop(reader_stream, reader_state))?;
        Ok(Self {
            name: name.to_string(),
            writer: Writer::new(stream),
            state,
            reader: Mutex::new(Some(reader)),
        })
    }

    /// Returns the registered peer name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Writes one envelope and returns its correlation id.
    ///
    /// # Arguments
    /// * `id` - Correlation id assigned by the caller.
    /// * `target` - Destination selector.
    /// * `kind` - Semantic class of the frame.
    /// * `body` - Serialized payload.
    ///
    /// # Returns
    /// The id used for the frame.
    ///
    /// # Errors
    /// Returns a codec or socket error.
    pub(crate) fn emit(
        &self,
        id: MessageId,
        target: Target,
        kind: Kind,
        body: serde_json::Value,
    ) -> io::Result<MessageId> {
        let envelope = Envelope::new(id, target, kind, body);
        self.writer.write_frame(&envelope)?;
        Ok(id)
    }

    /// Moves expired requests into the inbox as timeout notifications.
    ///
    /// # Arguments
    /// * `messages` - Inbox queue to append timeout notifications to.
    pub(crate) fn prune(&self, messages: &mut std::collections::VecDeque<Incoming>) {
        let now = Instant::now();
        let mut pending = lock(&self.state.pending);
        let expired: Vec<(MessageId, PeerName)> = pending
            .iter()
            .filter(|(_, entry)| entry.deadline <= now)
            .map(|(id, entry)| (*id, entry.target.clone()))
            .collect();
        for (id, target) in expired {
            pending.remove(&id);
            messages.push_back(Incoming::Timeout { id, target });
        }
    }

    /// Closes both ends of the socket and stops the reader thread.
    pub fn shutdown(&self) {
        self.state
            .stop
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.writer.shutdown();
        self.state.inbox.wake.notify_all();
        if let Some(handle) = lock(&self.reader).take() {
            let _ = handle.join();
        }
    }
}
