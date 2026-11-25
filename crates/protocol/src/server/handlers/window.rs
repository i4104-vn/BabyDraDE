//! Handles window lifecycle requests.

use crate::{
    models::{MessageId, ProtocolError, Request, Response, WindowAction},
    server::dispatcher::RequestHandler,
};

/// Adapts a compositor callback into a window request handler.
#[derive(Debug)]
pub struct WindowHandler<F> {
    apply: F,
}

impl<F> WindowHandler<F> {
    /// Builds a handler around `apply`.
    ///
    /// # Arguments
    /// * `apply` - Called with the requested action; returns `true` on success.
    pub fn new(apply: F) -> Self {
        Self { apply }
    }
}

impl<F> RequestHandler for WindowHandler<F>
where
    F: FnMut(WindowAction) -> bool,
{
    fn handle(&mut self, id: MessageId, request: Request) -> Response {
        match request {
            Request::WindowAction { action } => {
                if (self.apply)(action) {
                    Response::Ok { request_id: id }
                } else {
                    Response::Error {
                        request_id: id,
                        error: ProtocolError::Failed("window action rejected".into()),
                    }
                }
            }
            _ => Response::Error {
                request_id: id,
                error: ProtocolError::UnsupportedRequest,
            },
        }
    }
}
