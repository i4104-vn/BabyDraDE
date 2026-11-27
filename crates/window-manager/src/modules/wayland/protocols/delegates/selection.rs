use std::{io::Write, os::unix::io::OwnedFd};

use smithay::{
    delegate_data_device,
    input::Seat,
    wayland::selection::{
        data_device::{
            ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler,
        },
        SelectionHandler, SelectionSource, SelectionTarget,
    },
};

use crate::app::state::BabydraState;

impl SelectionHandler for BabydraState {
    type SelectionUserData = ();

    /// A client copied something. Defer the capture until Smithay has stored
    /// the source, then keep the data in our own cache.
    fn new_selection(
        &mut self,
        ty: SelectionTarget,
        source: Option<SelectionSource>,
        _seat: Seat<Self>,
    ) {
        if ty == SelectionTarget::Clipboard {
            if let Some(source) = source {
                self.note_client_selection(source.mime_types());
            }
        }
    }

    /// A client is pasting from the compositor-owned selection.
    fn send_selection(
        &mut self,
        ty: SelectionTarget,
        mime_type: String,
        fd: OwnedFd,
        _seat: Seat<Self>,
        _user_data: &Self::SelectionUserData,
    ) {
        if ty != SelectionTarget::Clipboard {
            return;
        }
        tracing::info!("Serving clipboard paste request for MIME '{mime_type}'");
        let Some(data) = services::system::clipboard::read_current_for(&mime_type) else {
            tracing::warn!("No cached clipboard data for MIME '{mime_type}'");
            return;
        };

        tracing::info!("Writing {} clipboard bytes to paste request", data.len());
        if let Err(err) = std::thread::Builder::new()
            .name("babydrade-clipboard-paste".to_string())
            .spawn(move || {
                let mut file = std::fs::File::from(fd);
                if let Err(err) = file.write_all(&data) {
                    tracing::warn!("Failed to serve clipboard selection: {err}");
                }
            })
        {
            tracing::warn!("Cannot spawn clipboard paste writer: {err}");
        }
    }
}

impl DataDeviceHandler for BabydraState {
    fn data_device_state(&self) -> &DataDeviceState {
        self.wayland.data_device_state()
    }
}

impl ClientDndGrabHandler for BabydraState {}
impl ServerDndGrabHandler for BabydraState {}

delegate_data_device!(BabydraState);
