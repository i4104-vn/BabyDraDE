//! Bridge between feature apps and the compositor.
//!
//! Features send typed [`Request`] values over the in-memory bus; this module
//! decodes one message, applies it to the live state and writes a [`Response`]
//! back through the compositor's local bus endpoint. All transport details stay
//! in [`crate::protocol`] — the compositor only sees plain data and the
//! handlers below.

use crate::protocol::{
    Message, MessageId, ProtocolError, Request, Response, WindowAction as ProtocolAction,
    WindowTarget,
};
use smithay::{desktop::Window, utils::SERIAL_COUNTER};

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
        Request::WindowAction { action } => handle_window(id, action, state),
        Request::CloseTargeted { target } => handle_close_targeted(id, target, state),
        Request::ShortcutRegister { owner, binding } => register_shortcut(id, owner, binding),
        Request::ShortcutUnregister {
            owner,
            id: binding_id,
        } => unregister_shortcut(id, owner, binding_id),
        Request::ShortcutList => list_shortcuts(id),
        Request::RunCommand { command } => handle_command(id, command, state),
        Request::SystemAction { target } => handle_system(id, target, state),
        Request::Ping => Response::Ok { request_id: id },
    }
}

/// Applies a window action to the compositor's active window.
///
/// # Arguments
/// * `id` - Correlation id of the originating request.
/// * `action` - Protocol action to translate and apply.
/// * `state` - Live compositor state holding the windows.
///
/// # Returns
/// An acknowledgement response.
fn handle_window(id: MessageId, action: ProtocolAction, state: &mut BabydraState) -> Response {
    BabydraState::handle_window_action(state, to_window_action(action));
    Response::Ok { request_id: id }
}

/// Closes the window matching a target selector.
///
/// # Arguments
/// * `id` - Correlation id of the originating request.
/// * `target` - Selector for the focused, app-id or title window.
/// * `state` - Live compositor state holding the windows.
///
/// # Returns
/// `Ok` when a window was closed, `NotFound` when none matched.
fn handle_close_targeted(
    id: MessageId,
    target: WindowTarget,
    state: &mut BabydraState,
) -> Response {
    match find_target(state, target) {
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

/// Finds the window matching a target selector.
///
/// # Arguments
/// * `state` - Live compositor state holding the windows.
/// * `target` - Selector for the focused, app-id or title window.
///
/// # Returns
/// A clone of the first matching window, if any.
fn find_target(state: &BabydraState, target: WindowTarget) -> Option<Window> {
    match target {
        WindowTarget::Focused => state.active_window(),
        WindowTarget::AppId(app_id) => state
            .windows
            .iter()
            .find(|window| metadata::get(window).is_some_and(|info| info.app_id == app_id))
            .cloned(),
        WindowTarget::Title(title) => state
            .windows
            .iter()
            .find(|window| metadata::get(window).is_some_and(|info| info.title.contains(&title)))
            .cloned(),
    }
}

/// Registers a shortcut binding for a feature owner.
///
/// # Arguments
/// * `id` - Correlation id of the originating request.
/// * `owner` - Stable id of the app owning the binding.
/// * `binding` - Binding to publish.
///
/// # Returns
/// `Ok` on success, or `Failed` carrying the registration error.
fn register_shortcut(id: MessageId, owner: String, binding: common::ShortcutBinding) -> Response {
    match crate::shortcuts::register(&owner, binding) {
        Ok(()) => Response::Ok { request_id: id },
        Err(error) => Response::Error {
            request_id: id,
            error: ProtocolError::Failed(error.to_string()),
        },
    }
}

/// Removes one shortcut binding by owner and id.
///
/// # Arguments
/// * `id` - Correlation id of the originating request.
/// * `owner` - Owner of the binding to remove.
/// * `binding_id` - Id of the binding to remove.
///
/// # Returns
/// `Ok` when a binding was removed, `NotFound` when none matched, or `Failed`.
fn unregister_shortcut(id: MessageId, owner: String, binding_id: String) -> Response {
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

/// Returns every registered shortcut binding with its owner.
///
/// # Arguments
/// * `id` - Correlation id of the originating request.
///
/// # Returns
/// A `Shortcuts` response, or `Failed` when the registry is unavailable.
fn list_shortcuts(id: MessageId) -> Response {
    match crate::shortcuts::bindings() {
        Ok(entries) => Response::Shortcuts {
            request_id: id,
            entries,
        },
        Err(error) => Response::Error {
            request_id: id,
            error: ProtocolError::Failed(error.to_string()),
        },
    }
}

/// Launches an external command through the compositor launcher.
///
/// # Arguments
/// * `id` - Correlation id of the originating request.
/// * `command` - Command string to execute.
/// * `state` - Live compositor state exposing the launcher.
///
/// # Returns
/// An acknowledgement response.
fn handle_command(id: MessageId, command: String, state: &mut BabydraState) -> Response {
    state.launch_command(command);
    Response::Ok { request_id: id }
}

/// Runs a built-in system action or command.
///
/// # Arguments
/// * `id` - Correlation id of the originating request.
/// * `target` - Built-in action or command string to trigger.
/// * `state` - Live compositor state the action acts on.
///
/// # Returns
/// An acknowledgement response.
fn handle_system(
    id: MessageId,
    target: common::ShortcutTarget,
    state: &mut BabydraState,
) -> Response {
    match target {
        common::ShortcutTarget::Command(command) => state.launch_command(command),
        common::ShortcutTarget::Action(action) => {
            execute_action(state, action, SERIAL_COUNTER.next_serial())
        }
    }
    Response::Ok { request_id: id }
}

/// Answers one request received from a feature peer.
///
/// # Arguments
/// * `message` - Incoming request envelope.
/// * `state` - Live compositor state the request acts on.
pub(crate) fn dispatch(message: Message, state: &mut BabydraState) {
    let response = match message.decode::<Request>() {
        Ok(request) => handle(message.id, request, state),
        Err(_) => Response::Error {
            request_id: message.id,
            error: ProtocolError::Failed("invalid request payload".into()),
        },
    };
    if let Err(error) = state.bus.respond(&message, &response) {
        tracing::warn!("failed to send protocol response: {error}");
    }
}
