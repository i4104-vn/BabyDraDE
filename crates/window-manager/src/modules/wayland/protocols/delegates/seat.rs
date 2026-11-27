use smithay::{
    delegate_cursor_shape, delegate_output, delegate_pointer_gestures, delegate_seat,
    input::{pointer::CursorImageStatus, Seat, SeatHandler, SeatState},
    reexports::wayland_server::{protocol::wl_surface::WlSurface, Resource},
    wayland::{
        output::OutputHandler, selection::data_device::set_data_device_focus,
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

impl OutputHandler for BabydraState {}

delegate_output!(BabydraState);

delegate_pointer_gestures!(BabydraState);
