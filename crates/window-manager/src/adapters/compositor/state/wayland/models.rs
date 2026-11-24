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
    pub(crate) _xdg_decoration_state: XdgDecorationState,
    pub(crate) wlr_layer_shell_state: WlrLayerShellState,
    pub(crate) _pointer_gestures_state: PointerGesturesState,
    /// Lets clients request named cursor shapes (text, wait, resize, ...).
    pub(crate) _cursor_shape_state: CursorShapeManagerState,
    pub(crate) shm_state: ShmState,
    pub(crate) _output_manager_state: OutputManagerState,
    pub(crate) seat_state: SeatState<BabydraState>,
    pub(crate) data_device_state: DataDeviceState,
    pub(crate) seat: Seat<BabydraState>,
}

