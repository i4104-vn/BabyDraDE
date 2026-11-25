//! Request dispatch: plug compositor-side handlers into the server.

use super::ProtocolServer;
use crate::models::{MessageId, Request, Response};

/// Something that can answer a protocol request.
pub trait RequestHandler {
    /// Handles one request, producing a response correlated by `id`.
    fn handle(&mut self, id: MessageId, request: Request) -> Response;
}

/// Drains requests from a server, runs the handler and writes responses.
#[derive(Debug)]
pub struct Dispatcher<H> {
    handler: H,
}

impl<H: RequestHandler> Dispatcher<H> {
    /// Builds a dispatcher around `handler`.
    pub fn new(handler: H) -> Self {
        Self { handler }
    }

    /// Polls the server once and answers every pending request.
    pub fn poll(&mut self, server: &ProtocolServer) {
        for (id, request) in server.drain_requests() {
            let response = self.handler.handle(id, request);
            let _ = server.respond(response);
        }
    }

    /// Returns a reference to the wrapped handler.
    pub fn handler(&self) -> &H {
        &self.handler
    }

    /// Returns a mutable reference to the wrapped handler.
    pub fn handler_mut(&mut self) -> &mut H {
        &mut self.handler
    }
}

impl<F> RequestHandler for F
where
    F: FnMut(MessageId, Request) -> Response,
{
    fn handle(&mut self, id: MessageId, request: Request) -> Response {
        self(id, request)
    }
}
