//! Bus connection state and the identifiers used to correlate requests.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use common::protocol::Incoming;

use crate::protocol::Bus;
use crate::system::shortcuts::error::ShortcutError;

use super::error::io_error;

/// Peer name the compositor registers on the bus.
pub(crate) const WM_PEER: &str = "wm";
/// Upper bound on how long one request waits for its response.
pub(super) const REPLY_TIMEOUT: Duration = Duration::from_secs(2);

/// Allocates distinct peer names so concurrent clients never collide.
static NEXT_PEER: AtomicU64 = AtomicU64::new(1);

/// Bus connection and responses received for requests still in flight.
#[derive(Default)]
pub(crate) struct ClientState {
    pub(crate) bus: Option<Bus>,
    pub(crate) deferred: VecDeque<Incoming>,
}

/// Locks the client state for one request.
///
/// # Arguments
/// * `state` - Mutex guarding the client state.
///
/// # Returns
/// A guard for the client state.
///
/// # Panics
/// Panics with `"client state lock poisoned"` when another thread panicked
/// while holding the lock; a poisoned state cannot be trusted.
pub(crate) fn lock_state(state: &Mutex<ClientState>) -> MutexGuard<'_, ClientState> {
    state.lock().expect("client state lock poisoned")
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
