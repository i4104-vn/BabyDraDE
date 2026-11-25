//! Compositor-side protocol server and dispatch machinery.

pub mod dispatcher;
pub mod handlers;

pub use dispatcher::{Dispatcher, RequestHandler};
pub use handlers::{NullHandler, ShortcutHandler, WindowHandler};

use crate::{
    codec::{decode, encode, CodecError},
    models::{
        ActiveWindowInfo, Envelope, Event, MessageId, Payload, ProtocolError, Request, Response,
    },
    transport::{paths, FileTransport, Transport},
};

/// Server side of the bridge, owned by the compositor.
#[derive(Debug)]
pub struct ProtocolServer {
    requests: FileTransport,
    responses: FileTransport,
    events: FileTransport,
    next_id: AtomicId,
}

#[derive(Debug)]
struct AtomicId(std::sync::atomic::AtomicU64);

impl ProtocolServer {
    /// Opens the server queues.
    pub fn new() -> Self {
        Self {
            requests: FileTransport::new(paths::REQUESTS_DIR),
            responses: FileTransport::new(paths::RESPONSES_DIR),
            events: FileTransport::new(paths::EVENTS_DIR),
            next_id: AtomicId(std::sync::atomic::AtomicU64::new(1)),
        }
    }

    fn allocate_id(&self) -> MessageId {
        self.next_id
            .0
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    fn write(&self, transport: &FileTransport, payload: Payload) -> Result<MessageId, CodecError> {
        let id = self.allocate_id();
        let bytes = encode(&Envelope::new(id, payload))?;
        transport
            .send(&bytes)
            .map_err(|error| CodecError::Io(error.to_string()))?;
        Ok(id)
    }

    /// Drains and decodes all pending requests.
    ///
    /// Malformed or version-mismatched envelopes are answered with an error
    /// response so the feature can notice the mismatch instead of hanging.
    pub fn drain_requests(&self) -> Vec<(MessageId, Request)> {
        let Ok(messages) = self.requests.drain() else {
            return Vec::new();
        };
        let mut requests = Vec::new();
        for bytes in &messages {
            match decode(bytes) {
                Ok(envelope) => {
                    if let Payload::Request(request) = envelope.payload {
                        requests.push((envelope.id, request));
                    }
                }
                Err(error) => {
                    let _ = self.respond(Response::Error {
                        request_id: 0,
                        error: match error {
                            CodecError::UnsupportedVersion { .. } => {
                                ProtocolError::UnsupportedVersion
                            }
                            _ => ProtocolError::Failed(error.to_string()),
                        },
                    });
                }
            }
        }
        requests
    }

    /// Sends a response for a request id.
    pub fn respond(&self, response: Response) -> Result<MessageId, CodecError> {
        self.write(&self.responses, Payload::Response(response))
    }

    /// Publishes an event to every listening feature.
    pub fn publish(&self, event: Event) -> Result<MessageId, CodecError> {
        self.write(&self.events, Payload::Event(event))
    }

    /// Publishes a focused-window snapshot.
    pub fn active_window_changed(&self, info: ActiveWindowInfo) -> Result<MessageId, CodecError> {
        self.publish(Event::ActiveWindowChanged { info })
    }
}

impl Default for ProtocolServer {
    fn default() -> Self {
        Self::new()
    }
}
