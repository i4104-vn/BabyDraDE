//! Bus connection, request correlation and response decoding helpers.

use std::collections::VecDeque;
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use common::protocol::{Incoming, Message, MessageId, ProtocolError, Response};

use crate::protocol::Bus;

use super::error::ShortcutError;

/// Peer name the compositor registers on the bus.
pub(crate) const WM_PEER: &str = "wm";
/// Upper bound on how long one request waits for its response.
const REPLY_TIMEOUT: Duration = Duration::from_secs(2);

/// Allocates distinct peer names so concurrent clients never collide.
static NEXT_PEER: AtomicU64 = AtomicU64::new(1);

/// Bus connection and responses received for requests still in flight.
#[derive(Default)]
pub(crate) struct ClientState {
    pub(crate) bus: Option<Bus>,
    pub(crate) deferred: VecDeque<Incoming>,
}

/// Locks the client state, recovering from a poisoned mutex.
///
/// # Arguments
/// * `state` - Mutex guarding the client state.
///
/// # Returns
/// A guard for the client state.
pub(crate) fn lock_state(state: &Mutex<ClientState>) -> MutexGuard<'_, ClientState> {
    state.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Returns whether an owner id can be sent to the compositor.
///
/// # Arguments
/// * `owner` - Candidate owner id.
///
/// # Returns
/// `false` for empty or whitespace-only ids.
pub(crate) fn valid_owner(owner: &str) -> bool {
    !owner.trim().is_empty()
}

/// Opens a bus connection for shortcut requests.
///
/// # Returns
/// A connected feature bus.
///
/// # Errors
/// Returns [`ShortcutError::Io`] when the router is unavailable or rejects
/// the connection.
pub(crate) fn connect_bus() -> Result<Bus, ShortcutError> {
    let serial = NEXT_PEER.fetch_add(1, Ordering::Relaxed);
    let name = format!("shortcuts.{}.{}", std::process::id(), serial);
    Bus::connect(&name).map_err(io_error)
}

/// Wraps a socket error as a shortcut failure.
///
/// # Arguments
/// * `error` - Error reported by the bus.
///
/// # Returns
/// A [`ShortcutError::Io`] carrying the error text.
pub(crate) fn io_error(error: io::Error) -> ShortcutError {
    ShortcutError::Io(error.to_string())
}

/// Lazily connects the client to the bus.
///
/// # Arguments
/// * `state` - Mutable client state holding the optional connection.
///
/// # Returns
/// A reference to the connected bus.
///
/// # Errors
/// Returns a connection failure when no bus is available.
pub(crate) fn ensure_bus(state: &mut ClientState) -> Result<&Bus, ShortcutError> {
    if state.bus.is_none() {
        state.bus = Some(connect_bus()?);
    }
    state
        .bus
        .as_ref()
        .ok_or_else(|| ShortcutError::Failed("shortcut bus is unavailable".to_string()))
}

/// Reports whether a response answers the request with `id`.
///
/// # Arguments
/// * `message` - Incoming response message.
/// * `id` - Correlation id of the request.
///
/// # Returns
/// `true` when the envelope id or its body identifies this request.
fn is_response_for(message: &Message, id: MessageId) -> bool {
    message.id == id
        || message
            .decode::<Response>()
            .map(|response| response.request_id() == id)
            .unwrap_or(false)
}

/// Decodes a response body into a typed response.
///
/// # Arguments
/// * `message` - Matching response message.
///
/// # Returns
/// The decoded response.
///
/// # Errors
/// Returns [`ShortcutError::InvalidManifest`] when the body is not a response.
fn decode_response(message: &Message) -> Result<Response, ShortcutError> {
    message
        .decode::<Response>()
        .map_err(|error| ShortcutError::InvalidManifest(error.to_string()))
}

/// Waits for the response correlated with `id`, retaining other messages.
///
/// # Arguments
/// * `bus` - Bus to receive from.
/// * `deferred` - Queue preserving messages for other correlation ids.
/// * `id` - Correlation id to wait for.
///
/// # Returns
/// The matching response.
///
/// # Errors
/// Returns a timeout, disconnect, bus error or malformed-response failure.
pub(crate) fn await_response(
    bus: &Bus,
    deferred: &mut VecDeque<Incoming>,
    id: MessageId,
) -> Result<Response, ShortcutError> {
    let deadline = Instant::now()
        .checked_add(REPLY_TIMEOUT)
        .unwrap_or_else(Instant::now);
    loop {
        let matched = deferred.iter().position(|incoming| match incoming {
            Incoming::Response(message) => is_response_for(message, id),
            _ => false,
        });
        if let Some(index) = matched {
            if let Some(Incoming::Response(message)) = deferred.remove(index) {
                return decode_response(&message);
            }
        }

        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(ShortcutError::Failed(format!(
                "timed out waiting for response to request {id}"
            )));
        }
        match bus.recv_timeout(remaining) {
            Some(Incoming::Response(message)) if is_response_for(&message, id) => {
                return decode_response(&message)
            }
            Some(Incoming::Timeout { id: timed, target }) if timed == id => Err(
                ShortcutError::Failed(format!("request {id} to {target} timed out")),
            ),
            Some(incoming @ Incoming::Error(_)) => {
                let detail = match &incoming {
                    Incoming::Error(error) => error.to_string(),
                    _ => String::new(),
                };
                deferred.push_back(incoming);
                Err(ShortcutError::Failed(detail))
            }
            Some(Incoming::Disconnected) => Err(ShortcutError::Failed(
                "shortcut bus disconnected".to_string(),
            )),
            Some(other) => {
                deferred.push_back(other);
                continue;
            }
            None => Err(ShortcutError::Failed(format!(
                "timed out waiting for response to request {id}"
            ))),
        }?;
    }
}

/// Converts an acknowledgement into a unit result.
///
/// # Arguments
/// * `response` - Response to interpret.
///
/// # Returns
/// `Ok(())` when the compositor acknowledged the request.
///
/// # Errors
/// Returns the compositor's failure or an unexpected response.
pub(crate) fn into_ack(response: Response) -> Result<(), ShortcutError> {
    match response {
        Response::Ok { .. } => Ok(()),
        Response::Error { error, .. } => Err(failure(&error)),
        other => Err(unexpected(&other)),
    }
}

/// Converts an unregister response into a removal outcome.
///
/// # Arguments
/// * `response` - Response to interpret.
///
/// # Returns
/// `false` when the binding was unknown, `true` when it was removed.
///
/// # Errors
/// Returns a compositor failure other than `NotFound`, or an unexpected response.
pub(crate) fn into_removal(response: Response) -> Result<bool, ShortcutError> {
    match response {
        Response::Ok { .. } => Ok(true),
        Response::Error {
            error: ProtocolError::NotFound,
            ..
        } => Ok(false),
        Response::Error { error, .. } => Err(failure(&error)),
        other => Err(unexpected(&other)),
    }
}

/// Extracts the binding table from a list response.
///
/// # Arguments
/// * `response` - Response to interpret.
///
/// # Returns
/// Every binding with its owner id.
///
/// # Errors
/// Returns a compositor failure or an unexpected response.
pub(crate) fn into_entries(
    response: Response,
) -> Result<Vec<(String, common::ShortcutBinding)>, ShortcutError> {
    match response {
        Response::Shortcuts { entries, .. } => Ok(entries),
        Response::Error { error, .. } => Err(failure(&error)),
        other => Err(unexpected(&other)),
    }
}

/// Maps a protocol failure to a shortcut failure.
///
/// # Arguments
/// * `error` - Failure reported by the compositor.
///
/// # Returns
/// A [`ShortcutError::Failed`] preserving the compositor's detail.
fn failure(error: &ProtocolError) -> ShortcutError {
    match error {
        ProtocolError::Failed(message) => ShortcutError::Failed(message.clone()),
        other => ShortcutError::Failed(format!("{other:?}")),
    }
}

/// Reports a response that does not fit the issued request.
///
/// # Arguments
/// * `response` - Unexpected response.
///
/// # Returns
/// A [`ShortcutError::Failed`] describing the response.
fn unexpected(response: &Response) -> ShortcutError {
    ShortcutError::Failed(format!("unexpected response: {response:?}"))
}
