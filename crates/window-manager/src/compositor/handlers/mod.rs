mod compositor;
mod layer_shell;
mod xdg_decoration;
mod xdg_shell;

use std::{io::Write, os::unix::io::OwnedFd, time::Duration};

use smithay::{
    delegate_data_device, delegate_output, delegate_pointer_gestures, delegate_seat,
    input::{pointer::CursorImageStatus, Seat, SeatHandler, SeatState},
    reexports::wayland_server::{protocol::wl_surface::WlSurface, Resource},
    wayland::{
        output::OutputHandler,
        selection::{
            data_device::{
                set_data_device_focus, ClientDndGrabHandler, DataDeviceHandler, DataDeviceState,
                ServerDndGrabHandler,
            },
            SelectionHandler, SelectionSource, SelectionTarget,
        },
    },
};

use super::state::BabydraState;

impl SeatHandler for BabydraState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: CursorImageStatus) {}

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        let client = focused.and_then(|surface| self.display_handle.get_client(surface.id()).ok());
        set_data_device_focus(&self.display_handle, seat, client);
    }
}

delegate_seat!(BabydraState);

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
        if let Some(data) =
            services::system::clipboard::wait_and_read_current(&mime_type, Duration::from_millis(500))
        {
            let mut file = std::fs::File::from(fd);
            if let Err(err) = file.write_all(&data) {
                tracing::warn!("Failed to serve clipboard selection: {err}");
            }
        }
    }
}

impl DataDeviceHandler for BabydraState {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl ClientDndGrabHandler for BabydraState {}
impl ServerDndGrabHandler for BabydraState {}

delegate_data_device!(BabydraState);

impl OutputHandler for BabydraState {}

delegate_output!(BabydraState);

delegate_pointer_gestures!(BabydraState);
