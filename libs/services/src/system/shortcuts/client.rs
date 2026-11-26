//! Bus-backed shortcut publisher.

use std::fmt;
use std::sync::Mutex;

use common::protocol::{Request, Response};
use common::{ShortcutBinding, ShortcutTarget};

use super::error::ShortcutError;
use super::snapshot::entries_for_owner;
use super::transport::{
    await_response, ensure_bus, into_ack, into_entries, into_removal, io_error, lock_state,
    valid_owner, ClientState, WM_PEER,
};

/// A shortcut publisher bound to one app id.
pub struct ShortcutClient {
    owner: String,
    state: Mutex<ClientState>,
}

impl fmt::Debug for ShortcutClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShortcutClient")
            .field("owner", &self.owner)
            .finish_non_exhaustive()
    }
}

impl Clone for ShortcutClient {
    /// Creates a sibling client with the same owner and a fresh bus connection.
    ///
    /// # Returns
    /// A client that connects lazily on its first operation.
    fn clone(&self) -> Self {
        Self::new(&self.owner)
    }
}

impl ShortcutClient {
    /// Creates a client for `owner`.
    ///
    /// # Arguments
    /// * `owner` - Stable app id, for example `org.babydrade.topbar`.
    ///
    /// # Returns
    /// A client whose operations all declare that owner to the compositor.
    pub fn new(owner: &str) -> Self {
        Self {
            owner: owner.to_string(),
            state: Mutex::new(ClientState::default()),
        }
    }

    /// Adds or replaces one binding for this client's owner.
    ///
    /// # Arguments
    /// * `binding` - Binding to publish; its `id` is the upsert key.
    ///
    /// # Returns
    /// `Ok(())` once the compositor acknowledges the registration.
    ///
    /// # Errors
    /// Returns [`ShortcutError::InvalidOwner`] for a blank owner, or a bus or
    /// compositor failure.
    pub fn register(&self, binding: ShortcutBinding) -> Result<(), ShortcutError> {
        if !valid_owner(&self.owner) {
            return Err(ShortcutError::InvalidOwner);
        }
        let request = Request::ShortcutRegister {
            owner: self.owner.clone(),
            binding,
        };
        into_ack(self.exchange(&request)?)
    }

    /// Registers a binding from its parts.
    ///
    /// # Arguments
    /// * `id` - Stable per-app shortcut id.
    /// * `combo` - Combo in keymap format, for example `W-S-t`.
    /// * `target` - Command or compositor action to run.
    ///
    /// # Returns
    /// `Ok(())` once the compositor acknowledges the registration.
    ///
    /// # Errors
    /// Returns a bus or compositor failure; see [`Self::register`].
    pub fn bind(&self, id: &str, combo: &str, target: ShortcutTarget) -> Result<(), ShortcutError> {
        self.register(ShortcutBinding {
            id: id.to_string(),
            combo: combo.to_string(),
            target,
            enabled: true,
        })
    }

    /// Removes one binding owned by this client.
    ///
    /// # Arguments
    /// * `id` - Id of the binding to remove.
    ///
    /// # Returns
    /// `true` when a binding was removed, `false` when the id was unknown.
    ///
    /// # Errors
    /// Returns [`ShortcutError::InvalidOwner`] for a blank owner, or a bus or
    /// compositor failure other than a missing binding.
    pub fn unregister(&self, id: &str) -> Result<bool, ShortcutError> {
        if !valid_owner(&self.owner) {
            return Err(ShortcutError::InvalidOwner);
        }
        let request = Request::ShortcutUnregister {
            owner: self.owner.clone(),
            id: id.to_string(),
        };
        into_removal(self.exchange(&request)?)
    }

    /// Removes every binding owned by this client.
    ///
    /// # Returns
    /// `Ok(())` even when this client has no bindings.
    ///
    /// # Errors
    /// Returns a bus or compositor failure while listing or unregistering.
    pub fn clear(&self) -> Result<(), ShortcutError> {
        if !valid_owner(&self.owner) {
            return Err(ShortcutError::InvalidOwner);
        }
        let bindings = entries_for_owner(
            into_entries(self.exchange(&Request::ShortcutList)?)?,
            &self.owner,
        );
        for binding in bindings {
            self.unregister(&binding.id)?;
        }
        Ok(())
    }

    /// Returns this client's currently registered bindings.
    ///
    /// # Returns
    /// An empty vector when nothing is registered or the bus cannot be read.
    pub fn bindings(&self) -> Vec<ShortcutBinding> {
        match self.exchange(&Request::ShortcutList).and_then(into_entries) {
            Ok(entries) => entries_for_owner(entries, &self.owner),
            Err(error) => {
                tracing::warn!("cannot list shortcuts for {}: {error}", self.owner);
                Vec::new()
            }
        }
    }

    /// Sends one request to the compositor and returns its correlated response.
    ///
    /// # Arguments
    /// * `request` - Request body to send.
    ///
    /// # Returns
    /// The decoded response assigned to this request's id.
    ///
    /// # Errors
    /// Returns a connection, timeout or malformed-response failure.
    fn exchange(&self, request: &Request) -> Result<Response, ShortcutError> {
        let mut state = lock_state(&self.state);
        let id = ensure_bus(&mut state)?
            .request(WM_PEER, request)
            .map_err(io_error)?;
        let mut deferred = std::mem::take(&mut state.deferred);
        let result = {
            let bus = state
                .bus
                .as_ref()
                .ok_or_else(|| ShortcutError::Failed("shortcut bus is unavailable".to_string()))?;
            await_response(bus, &mut deferred, id)
        };
        state.deferred = deferred;
        result
    }
}
