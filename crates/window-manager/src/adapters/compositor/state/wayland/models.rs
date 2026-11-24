//! Wayland protocol state models.

use smithay::{
    input::{Seat, SeatState},
    reexports::wayland_server::DisplayHandle,
    wayland::{
        compositor::CompositorState,
        cursor_shape::CursorShapeManagerState,
        output::OutputManagerState,
        pointer_gestures::PointerGesturesState,
        selection::data_device::DataDeviceState,
        shell::{
            wlr_layer::WlrLayerShellState,
            xdg::{decoration::XdgDecorationState, XdgShellState},
        },
        shm::ShmState,
    },
};

use crate::adapters::compositor::BabydraState;

/// All long-lived Wayland protocol state for one compositor instance.
pub(crate) struct WaylandRuntime {
    pub(crate) display_handle: DisplayHandle,
    pub(crate) compositor_state: CompositorState,
    pub(crate) xdg_shell_state: XdgShellState,
    pub(crate) wlr_layer_shell_state: WlrLayerShellState,
    pub(crate) shm_state: ShmState,
    /// Protocol globals advertised without dedicated handlers.
    pub(crate) _extensions: ProtocolExtensions,
    /// The compositor's single seat and its input/selection state.
    pub(crate) seat: SeatRuntime,
}

/// Protocol globals that must stay alive for the compositor's lifetime.
pub(crate) struct ProtocolExtensions {
    pub(crate) _xdg_decoration_state: XdgDecorationState,
    pub(crate) _pointer_gestures_state: PointerGesturesState,
    /// Lets clients request named cursor shapes (text, wait, resize, ...).
    pub(crate) _cursor_shape_state: CursorShapeManagerState,
    pub(crate) _output_manager_state: OutputManagerState,
}

/// The seat plus the state required by Smithay's input and selection delegates.
pub(crate) struct SeatRuntime {
    pub(crate) seat: Seat<BabydraState>,
    pub(crate) seat_state: SeatState<BabydraState>,
    pub(crate) data_device_state: DataDeviceState,
}

impl WaylandRuntime {
    /// Returns the compositor's single Wayland seat.
    pub(crate) fn seat(&self) -> &Seat<BabydraState> {
        &self.seat.seat
    }

    /// Returns the seat state used by the Smithay input delegates.
    pub(crate) fn seat_state_mut(&mut self) -> &mut SeatState<BabydraState> {
        &mut self.seat.seat_state
    }

    /// Returns the data device state used by the Smithay selection delegates.
    pub(crate) fn data_device_state(&self) -> &DataDeviceState {
        &self.seat.data_device_state
    }
}
