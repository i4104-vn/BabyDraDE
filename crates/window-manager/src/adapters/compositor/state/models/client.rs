//! Wayland client data attached to each connected client.

use smithay::{
    reexports::wayland_server::backend::{ClientData, ClientId, DisconnectReason},
    wayland::compositor::CompositorClientState,
};

#[derive(Default)]
pub(crate) struct ClientState {
    pub(crate) compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}

    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}
