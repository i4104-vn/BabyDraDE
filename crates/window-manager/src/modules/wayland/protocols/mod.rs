
use std::{io::Write, os::unix::io::OwnedFd};

use smithay::{
    delegate_cursor_shape, delegate_data_device, delegate_output, delegate_pointer_gestures,
    delegate_seat,
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
        tablet_manager::TabletSeatHandler,
    },
};

use crate::app::state::BabydraState;

impl SeatHandler for BabydraState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        self.wayland.seat_state_mut()
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, image: CursorImageStatus) {
        match image {
            CursorImageStatus::Hidden => self.input.cursor_visible = false,
            CursorImageStatus::Named(icon) => {
                self.input.cursor_icon = icon;
                self.input.cursor_visible = true;
            }
            // Client-supplied cursor surfaces are not part of a theme; keep the
            // last themed shape visible instead of dropping the pointer.
            CursorImageStatus::Surface(_) => self.input.cursor_visible = true,
        }
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        let client =
            focused.and_then(|surface| self.wayland.display_handle.get_client(surface.id()).ok());
        set_data_device_focus(&self.wayland.display_handle, seat, client);
    }
}

delegate_seat!(BabydraState);
delegate_cursor_shape!(BabydraState);

// Required by the cursor-shape delegate; the tablet protocol itself is not
// advertised, so there are no tool images to handle.
impl TabletSeatHandler for BabydraState {}

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

impl OutputHandler for BabydraState {}

delegate_output!(BabydraState);

delegate_pointer_gestures!(BabydraState);

mod compositor;
mod layer_shell;
mod xdg_decoration;
mod xdg_shell;
