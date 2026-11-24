//! Wayland globals and seat construction.

use smithay::{
    input::SeatState,
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

use super::models::{ProtocolExtensions, SeatRuntime, WaylandRuntime};
use crate::adapters::compositor::BabydraState;

impl WaylandRuntime {
    /// Creates the Smithay globals and the single seat used by BabyDraDE.
    pub(crate) fn new(display_handle: &DisplayHandle) -> Self {
        let compositor_state = CompositorState::new::<BabydraState>(display_handle);
        let xdg_shell_state = XdgShellState::new::<BabydraState>(display_handle);
        let xdg_decoration_state = XdgDecorationState::new::<BabydraState>(display_handle);
        let wlr_layer_shell_state = WlrLayerShellState::new::<BabydraState>(display_handle);
        let pointer_gestures_state = PointerGesturesState::new::<BabydraState>(display_handle);
        let cursor_shape_state = CursorShapeManagerState::new::<BabydraState>(display_handle);
        let shm_state = ShmState::new::<BabydraState>(display_handle, vec![]);
        let output_manager_state =
            OutputManagerState::new_with_xdg_output::<BabydraState>(display_handle);
        let data_device_state = DataDeviceState::new::<BabydraState>(display_handle);
        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(display_handle, "babydrade");

        // The nested Winit window provides a keyboard and pointer for this first version.
        // Startup fail-fast: without a keyboard capability the seat cannot
        // deliver key events, so surface the cause instead of running degraded.
        seat.add_keyboard(Default::default(), 200, 25)
            .expect("failed to initialize the babydrade seat keyboard capability");
        seat.add_pointer();
        seat.add_touch();

        Self {
            display_handle: display_handle.clone(),
            compositor_state,
            xdg_shell_state,
            wlr_layer_shell_state,
            shm_state,
            _extensions: ProtocolExtensions {
                _xdg_decoration_state: xdg_decoration_state,
                _pointer_gestures_state: pointer_gestures_state,
                _cursor_shape_state: cursor_shape_state,
                _output_manager_state: output_manager_state,
            },
            seat: SeatRuntime {
                seat,
                seat_state,
                data_device_state,
            },
        }
    }
}
