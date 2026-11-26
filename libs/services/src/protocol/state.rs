//! Reader-thread state and frame routing for the feature bus.

use std::collections::{HashMap, VecDeque};
use std::io::{self, ErrorKind, Read};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, PoisonError, RwLock};
use std::time::{Duration, Instant};

use common::protocol::writer::lock;
use common::protocol::{
    decode, BusError, BusErrorCode, Control, Envelope, Incoming, Kind, LineBuffer, Message,
    MessageId, PeerName, Target,
};
// Router/client deadline and presence topic are owned by `common::protocol`.
pub(crate) use common::protocol::{PEERS_TOPIC, REQUEST_TIMEOUT};

/// Deadline for the initial `Hello`/`Welcome` exchange.
pub(crate) const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);
/// Size of each socket read chunk.
pub(crate) const CHUNK: usize = 4096;

/// Deadline and destination of an unanswered request.
pub(crate) struct ReaderPending {
    pub(crate) deadline: Instant,
    pub(crate) target: PeerName,
}

/// Inbox queue with a condvar for blocking receives.
pub(crate) struct Inbox {
    pub(crate) messages: Mutex<VecDeque<Incoming>>,
    pub(crate) wake: Condvar,
}

impl Inbox {
    /// Appends a message and wakes one blocked receiver.
    ///
    /// # Arguments
    /// * `incoming` - Message to queue.
    pub(crate) fn push(&self, incoming: Incoming) {
        let mut messages = lock(&self.messages);
        messages.push_back(incoming);
        self.wake.notify_one();
    }
}

/// State shared with the background reader thread.
pub(crate) struct BusState {
    pub(crate) name: PeerName,
    pub(crate) inbox: Inbox,
    pub(crate) peers: RwLock<Vec<PeerName>>,
    pub(crate) pending: Mutex<HashMap<MessageId, ReaderPending>>,
    pub(crate) stop: AtomicBool,
}

/// Merges the router's peer list with the local name, sorted and deduplicated.
///
/// # Arguments
/// * `name` - Local peer name to include.
/// * `peers` - Peers reported by the router.
///
/// # Returns
/// The canonical visible-peer list.
pub(crate) fn with_self(name: &str, mut peers: Vec<PeerName>) -> Vec<PeerName> {
    if !peers.iter().any(|peer| peer == name) {
        peers.push(name.to_string());
    }
    peers.sort();
    peers.dedup();
    peers
}

/// Applies a control frame to the visible-peer list.
///
/// # Arguments
/// * `state` - Shared bus state to update.
/// * `control` - Control frame to apply.
pub(crate) fn apply_control(state: &BusState, control: &Control) {
    let mut peers = state.peers.write().unwrap_or_else(PoisonError::into_inner);
    match control {
        Control::Welcome { peers: list, .. } => *peers = with_self(&state.name, list.clone()),
        Control::PeerJoined { name } => {
            peers.push(name.clone());
            peers.sort();
            peers.dedup();
        }
        Control::PeerLeft { name } => peers.retain(|peer| peer != name),
        _ => {}
    }
}

/// Turns an envelope into an incoming message, or `None` for a control frame.
///
/// # Arguments
/// * `envelope` - Decoded frame from the socket.
///
/// # Returns
/// The classified message, when the frame carries one.
pub(crate) fn deliver(envelope: Envelope) -> Option<Incoming> {
    match envelope.kind {
        Kind::Error => Some(Incoming::Error(envelope.bus_error().unwrap_or_else(|| {
            BusError::new(BusErrorCode::InvalidPayload, "malformed error frame")
        }))),
        Kind::Response => Some(Incoming::Response(Message::from_envelope(envelope))),
        Kind::Request => Some(Incoming::Request(Message::from_envelope(envelope))),
        Kind::Event => Some(Incoming::Event(Message::from_envelope(envelope))),
    }
}

/// Routes one envelope to the peer cache and the client inbox.
///
/// # Arguments
/// * `envelope` - Decoded frame from the socket.
/// * `state` - Shared bus state to update and enqueue into.
pub(crate) fn route_frame(envelope: Envelope, state: &BusState) {
    if envelope.target == Target::Router {
        if let Some(control) = envelope.control() {
            apply_control(state, &control);
        }
        return;
    }
    if matches!(envelope.kind, Kind::Error | Kind::Response) {
        lock(&state.pending).remove(&envelope.id);
    }
    if matches!(&envelope.target, Target::Topic(topic) if topic == PEERS_TOPIC) {
        if let Some(control) = envelope.control() {
            apply_control(state, &control);
        }
    }
    if let Some(incoming) = deliver(envelope) {
        state.inbox.push(incoming);
    }
}

/// Decodes and routes one raw frame, reporting parse failures to the inbox.
///
/// # Arguments
/// * `line` - Raw bytes of one frame, without the newline.
/// * `state` - Shared bus state to update and enqueue into.
pub(crate) fn handle_frame(line: &[u8], state: &BusState) {
    let envelope = match decode(line) {
        Ok(envelope) => envelope,
        Err(error) => {
            state.inbox.push(Incoming::Error(BusError::new(
                BusErrorCode::InvalidPayload,
                error.to_string(),
            )));
            return;
        }
    };
    route_frame(envelope, state);
}

/// Reads frames until the socket closes or the bus stops.
///
/// # Arguments
/// * `stream` - Socket dedicated to reading.
/// * `state` - Shared bus state updated by incoming frames.
pub(crate) fn reader_loop(mut stream: UnixStream, state: Arc<BusState>) {
    let (mut buffer, mut chunk) = (LineBuffer::new(), [0u8; CHUNK]);
    while !state.stop.load(Ordering::Relaxed) {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => {
                if let Err(error) = buffer.push(&chunk[..count]) {
                    let error = BusError::new(BusErrorCode::TooLarge, error.to_string());
                    state.inbox.push(Incoming::Error(error));
                    break;
                }
                while let Some(line) = buffer.take_line() {
                    handle_frame(&line, &state);
                }
            }
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(_) => break,
        }
    }
    if !state.stop.load(Ordering::Relaxed) {
        state.inbox.push(Incoming::Disconnected);
    }
}

/// Reads frames until the router answers `Hello` with `Welcome`, keeping any
/// early frames for the reader thread.
///
/// # Arguments
/// * `stream` - Socket positioned immediately after the `Hello` frame.
///
/// # Returns
/// The router's peer list and the early peer-directed frames.
///
/// # Errors
/// Returns a socket or codec error, `PermissionDenied` when the name is
/// rejected, or `TimedOut` when the welcome never arrives.
pub(crate) fn await_welcome(
    stream: &mut UnixStream,
) -> io::Result<(Vec<PeerName>, VecDeque<Incoming>)> {
    let (mut buffer, mut chunk) = (LineBuffer::new(), [0u8; CHUNK]);
    let (mut early, mut welcome) = (VecDeque::new(), None);
    while welcome.is_none() {
        match stream.read(&mut chunk) {
            Ok(0) => {
                return Err(io::Error::new(
                    ErrorKind::UnexpectedEof,
                    "bus closed during handshake",
                ))
            }
            Ok(count) => {
                if let Err(error) = buffer.push(&chunk[..count]) {
                    return Err(io::Error::new(ErrorKind::InvalidData, error));
                }
                while let Some(line) = buffer.take_line() {
                    let envelope = decode(&line)
                        .map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
                    if envelope.kind == Kind::Error {
                        let error = envelope.bus_error().unwrap_or_else(|| {
                            BusError::new(BusErrorCode::InvalidPayload, "malformed handshake error")
                        });
                        return Err(io::Error::new(ErrorKind::PermissionDenied, error));
                    }
                    if envelope.target == Target::Router {
                        if let Some(Control::Welcome { peers, .. }) = envelope.control() {
                            welcome = Some(peers);
                        }
                    } else if let Some(incoming) = deliver(envelope) {
                        early.push_back(incoming);
                    }
                }
            }
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) if matches!(error.kind(), ErrorKind::TimedOut | ErrorKind::WouldBlock) => {
                return Err(io::Error::new(
                    ErrorKind::TimedOut,
                    "timed out waiting for bus welcome",
                ))
            }
            Err(error) => return Err(error),
        }
    }
    Ok((welcome.unwrap_or_default(), early))
}
