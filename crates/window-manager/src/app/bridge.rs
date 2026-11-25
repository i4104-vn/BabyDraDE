//! Bridge between feature apps and the compositor.
//!
//! Features publish typed [`Request`] values; this module drains them once per
//! frame, applies them to the live state and writes [`Response`] values back.
//! All transport details stay in the `protocol` crate — the compositor only
//! sees plain data and the handlers below.

use protocol::{
    models::{MessageId, ProtocolError, Request, Response},
    server::{Dispatcher, ProtocolServer},
    WindowAction as ProtocolAction, WindowTarget,
};

use smithay::utils::SERIAL_COUNTER;

use crate::{
    app::state::BabydraState,
    modules::{
        input::keyboard::shortcuts::execute_action,
        window::{metadata, windows::WindowAction},
    },
    shared::ports::SystemActions,
};

/// Maps a protocol action to the compositor's internal window action.
fn to_window_action(action: ProtocolAction) -> WindowAction {
    match action {
        ProtocolAction::Close => WindowAction::Close,
        ProtocolAction::Minimize => WindowAction::Minimize,
        ProtocolAction::Maximize => WindowAction::Maximize,
        ProtocolAction::Fullscreen => WindowAction::Fullscreen,
    }
}

/// Handles one request against live compositor state.
fn handle(id: MessageId, request: Request, state: &mut BabydraState) -> Response {
    match request {
        Request::WindowAction { action } => {
            BabydraState::handle_window_action(state, to_window_action(action));
            Response::Ok { request_id: id }
        }
        Request::CloseTargeted { target } => {
            let window = match target {
                WindowTarget::Focused => state.active_window(),
                WindowTarget::AppId(app_id) => state
                    .windows
                    .iter()
                    .find(|window| {
                        metadata::get(window).is_some_and(|info| info.app_id == app_id)
                    })
                    .cloned(),
                WindowTarget::Title(title) => state
                    .windows
                    .iter()
                    .find(|window| {
                        metadata::get(window)
                            .is_some_and(|info| info.title.contains(&title))
                    })
                    .cloned(),
            };
            match window {
                Some(window) => {
                    state.close_window(&window);
                    Response::Ok { request_id: id }
                }
                None => Response::Error {
                    request_id: id,
                    error: ProtocolError::NotFound,
                },
            }
        }
        Request::ShortcutRegister { owner, binding } => {
            match crate::shortcuts::register(&owner, binding) {
                Ok(()) => Response::Ok { request_id: id },
                Err(error) => Response::Error {
                    request_id: id,
                    error: ProtocolError::Failed(error.to_string()),
                },
            }
        }
        Request::ShortcutUnregister { owner, id: binding_id } => {
            match crate::shortcuts::unregister(&owner, &binding_id) {
                Ok(true) => Response::Ok { request_id: id },
                Ok(false) => Response::Error {
                    request_id: id,
                    error: ProtocolError::NotFound,
                },
                Err(error) => Response::Error {
                    request_id: id,
                    error: ProtocolError::Failed(error.to_string()),
                },
            }
        }
        Request::ShortcutList => match crate::shortcuts::bindings() {
            Ok(entries) => Response::Shortcuts {
                request_id: id,
                entries,
            },
            Err(error) => Response::Error {
                request_id: id,
                error: ProtocolError::Failed(error.to_string()),
            },
        },
        Request::RunCommand { command } => {
            state.launch_command(command);
            Response::Ok { request_id: id }
        }
        Request::SystemAction { target } => {
            match target {
                common::ShortcutTarget::Command(command) => state.launch_command(command),
                common::ShortcutTarget::Action(action) => execute_action(
                    state,
                    action,
                    SERIAL_COUNTER.next_serial(),
                ),
            }
            Response::Ok { request_id: id }
        }
        Request::Ping => Response::Ok { request_id: id },
    }
}

/// Polls the bridge once. Call from the frame loop.
pub(crate) fn poll(state: &mut BabydraState) {
    let server = ProtocolServer::new();
    let mut dispatcher = Dispatcher::new(|id, request| handle(id, request, state));
    dispatcher.poll(&server);
}
