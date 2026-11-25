//! Handles shortcut registration requests.

use common::ShortcutBinding;

use crate::{
    models::{MessageId, ProtocolError, Request, Response},
    server::dispatcher::RequestHandler,
};

type RegisterFn = Box<dyn FnMut(String, ShortcutBinding) -> bool>;
type UnregisterFn = Box<dyn FnMut(String, String) -> bool>;
type ListFn = Box<dyn FnMut() -> Vec<(String, ShortcutBinding)>>;

/// Adapts the compositor's shortcut registry into a request handler.
pub struct ShortcutHandler {
    register: RegisterFn,
    unregister: UnregisterFn,
    list: ListFn,
}

impl std::fmt::Debug for ShortcutHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ShortcutHandler { .. }")
    }
}

impl ShortcutHandler {
    /// Builds a handler from three registry callbacks.
    pub fn new(
        register: RegisterFn,
        unregister: UnregisterFn,
        list: ListFn,
    ) -> Self {
        Self {
            register,
            unregister,
            list,
        }
    }
}

impl RequestHandler for ShortcutHandler {
    fn handle(&mut self, id: MessageId, request: Request) -> Response {
        match request {
            Request::ShortcutRegister { owner, binding } => {
                if (self.register)(owner, binding) {
                    Response::Ok { request_id: id }
                } else {
                    Response::Error {
                        request_id: id,
                        error: ProtocolError::Failed("shortcut registration rejected".into()),
                    }
                }
            }
            Request::ShortcutUnregister { owner, id: binding_id } => {
                if (self.unregister)(owner, binding_id) {
                    Response::Ok { request_id: id }
                } else {
                    Response::Error {
                        request_id: id,
                        error: ProtocolError::NotFound,
                    }
                }
            }
            Request::ShortcutList => Response::Shortcuts {
                request_id: id,
                entries: (self.list)(),
            },
            _ => Response::Error {
                request_id: id,
                error: ProtocolError::UnsupportedRequest,
            },
        }
    }
}
