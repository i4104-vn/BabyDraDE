//! Correlating responses with requests and decoding their bodies.

use std::collections::VecDeque;
use std::time::Instant;

use common::protocol::{Incoming, Message, MessageId, ProtocolError, Response};

use crate::protocol::Bus;
use crate::system::shortcuts::error::ShortcutError;

use super::error::{failure, unexpected};
use super::state::REPLY_TIMEOUT;

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
