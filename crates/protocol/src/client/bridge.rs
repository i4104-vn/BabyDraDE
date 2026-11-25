//! Feature-side bridge: send requests and consume events/responses.

use std::sync::atomic::{AtomicU64, Ordering};

use common::ShortcutBinding;

use crate::{
    codec::{decode, encode, CodecError},
    models::{Envelope, Event, MessageId, Payload, Request, Response, WindowAction, WindowTarget},
    transport::{paths, FileTransport, Transport},
};

/// Client used by a feature app to talk to the compositor.
///
/// One bridge per process; the `owner` id namespaces shortcut bindings.
#[derive(Debug)]
pub struct FeatureBridge {
    owner: String,
    requests: FileTransport,
    responses: FileTransport,
    events: FileTransport,
    next_id: AtomicU64,
}

impl FeatureBridge {
    /// Opens the bridge queues for `owner`.
    ///
    /// # Arguments
    /// * `owner` - Stable feature id, for example `org.babydrade.topbar`.
    pub fn new(owner: &str) -> Self {
        Self {
            owner: owner.to_string(),
            requests: FileTransport::new(paths::REQUESTS_DIR),
            responses: FileTransport::new(paths::RESPONSES_DIR),
            events: FileTransport::new(paths::EVENTS_DIR),
            next_id: AtomicU64::new(1),
        }
    }

    /// Returns the owner id of this bridge.
    pub fn owner(&self) -> &str {
        &self.owner
    }

    fn allocate_id(&self) -> MessageId {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Sends a request and returns its correlation id.
    ///
    /// # Arguments
    /// * `request` - Operation to perform.
    pub fn send_request(&self, request: Request) -> Result<MessageId, CodecError> {
        let id = self.allocate_id();
        let bytes = encode(&Envelope::new(id, Payload::Request(request)))?;
        self.requests
            .send(&bytes)
            .map_err(|error| CodecError::Io(error.to_string()))?;
        Ok(id)
    }

    /// Returns all responses addressed to this process since the last call.
    pub fn drain_responses(&self) -> Vec<Response> {
        let Ok(messages) = self.responses.drain() else {
            return Vec::new();
        };
        messages
            .iter()
            .filter_map(|bytes| decode(bytes).ok())
            .filter_map(|envelope| match envelope.payload {
                Payload::Response(response) => Some(response),
                _ => None,
            })
            .collect()
    }

    /// Returns all compositor events since the last call.
    pub fn poll_events(&self) -> Vec<Event> {
        let Ok(messages) = self.events.drain() else {
            return Vec::new();
        };
        messages
            .iter()
            .filter_map(|bytes| decode(bytes).ok())
            .filter_map(|envelope| match envelope.payload {
                Payload::Event(event) => Some(event),
                _ => None,
            })
            .collect()
    }

    /// Requests a window lifecycle action.
    pub fn window_action(&self, action: WindowAction) -> Result<MessageId, CodecError> {
        self.send_request(Request::WindowAction { action })
    }

    /// Requests closing a specific window.
    pub fn close_targeted(&self, target: WindowTarget) -> Result<MessageId, CodecError> {
        self.send_request(Request::CloseTargeted { target })
    }

    /// Publishes one shortcut binding owned by this feature.
    pub fn register_shortcut(&self, binding: ShortcutBinding) -> Result<MessageId, CodecError> {
        self.send_request(Request::ShortcutRegister {
            owner: self.owner.clone(),
            binding,
        })
    }

    /// Removes one shortcut binding by id.
    pub fn unregister_shortcut(&self, id: &str) -> Result<MessageId, CodecError> {
        self.send_request(Request::ShortcutUnregister {
            owner: self.owner.clone(),
            id: id.to_string(),
        })
    }

    /// Requests the full shortcut table.
    pub fn list_shortcuts(&self) -> Result<MessageId, CodecError> {
        self.send_request(Request::ShortcutList)
    }

    /// Requests a shell command launch.
    pub fn run_command(&self, command: &str) -> Result<MessageId, CodecError> {
        self.send_request(Request::RunCommand {
            command: command.to_string(),
        })
    }

    /// Sends a liveness probe.
    pub fn ping(&self) -> Result<MessageId, CodecError> {
        self.send_request(Request::Ping)
    }
}
