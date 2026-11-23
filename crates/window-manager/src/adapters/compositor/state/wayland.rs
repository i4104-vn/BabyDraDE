//! Smithay protocol globals and seat runtime owned by the compositor.

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
        seat.add_keyboard(Default::default(), 200, 25)
            .expect("failed to initialize keyboard");
        seat.add_pointer();
        seat.add_touch();

        Self {
            display_handle: display_handle.clone(),
            compositor_state,
            xdg_shell_state,
            _xdg_decoration_state: xdg_decoration_state,
            wlr_layer_shell_state,
            _pointer_gestures_state: pointer_gestures_state,
            _cursor_shape_state: cursor_shape_state,
            shm_state,
            _output_manager_state: output_manager_state,
            seat_state,
            data_device_state,
            seat,
        }
    }
}
