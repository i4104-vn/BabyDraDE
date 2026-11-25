//! Reusable request handlers, split by capability.
//!
//! Adding a new feature to the protocol means adding one file here and one
//! request variant in `models/request.rs`; existing handlers stay untouched.

pub mod shortcut;
pub mod window;

pub use shortcut::ShortcutHandler;
pub use window::WindowHandler;

use crate::models::{MessageId, ProtocolError, Request, Response};

/// Handler that rejects every request with `UnsupportedRequest`.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullHandler;

impl super::dispatcher::RequestHandler for NullHandler {
    fn handle(&mut self, id: MessageId, _request: Request) -> Response {
        Response::Error {
            request_id: id,
            error: ProtocolError::UnsupportedRequest,
        }
    }
}

/// Tries `first`, then `second` when the first handler does not support the
/// request. Keeps each capability handler free of knowledge about the others.
#[derive(Debug)]
pub struct ChainHandler<A, B> {
    first: A,
    second: B,
}

impl<A, B> ChainHandler<A, B> {
    /// Chains two handlers.
    pub fn new(first: A, second: B) -> Self {
        Self { first, second }
    }
}

impl<A: super::dispatcher::RequestHandler, B: super::dispatcher::RequestHandler>
    super::dispatcher::RequestHandler for ChainHandler<A, B>
{
    fn handle(&mut self, id: MessageId, request: Request) -> Response {
        let response = self.first.handle(id, request.clone());
        if matches!(response, Response::Error { error: ProtocolError::UnsupportedRequest, .. }) {
            self.second.handle(id, request)
        } else {
            response
        }
    }
}
