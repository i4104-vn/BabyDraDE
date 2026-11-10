use std::{collections::HashMap, ffi::OsString, sync::Arc, time::Instant};

use smithay::{
    desktop::{PopupManager, Space, Window},
    input::{Seat, SeatState},
    reexports::{
        calloop::{
            generic::Generic, EventLoop, Interest, LoopSignal, Mode as CalloopMode, PostAction,
        },
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            Display, DisplayHandle,
        },
    },
    utils::{Logical, Point},
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        output::OutputManagerState,
        pointer_gestures::PointerGesturesState,
        selection::data_device::DataDeviceState,
        shell::{wlr_layer::WlrLayerShellState, xdg::XdgShellState},
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};

mod active_window;
mod blur;
mod hit_test;
mod switcher;
mod window_ops;

pub(crate) use switcher::{AltTabEntry, AltTabSwitcher};

pub(crate) struct CalloopData {
    pub(crate) state: BabydraState,
    pub(crate) display_handle: DisplayHandle,
}

#[derive(Debug, Clone)]
pub(crate) struct MoveState {
    pub(crate) window: Window,
    pub(crate) initial_window_location: Point<i32, Logical>,
    pub(crate) initial_pointer_location: Point<f64, Logical>,
    pub(crate) has_dragged: bool,
    pub(crate) button: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct TrackpadSwipeBuffer {
    pub(crate) fingers: u32,
    pub(crate) time: u32,
    pub(crate) total_delta: Point<f64, Logical>,
    pub(crate) updates: Vec<(Point<f64, Logical>, u32)>,
}

/// Direction resolved from an accumulated gesture delta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}

impl SwipeDirection {
    /// Resolves the dominant direction from a 2-D accumulated delta.
    ///
    /// Returns `None` when neither axis exceeds `threshold`.
    ///
    /// # Arguments
    ///
    /// * `dx` - Total horizontal displacement (positive = right).
    /// * `dy` - Total vertical displacement (positive = down).
    /// * `threshold` - Minimum absolute displacement to consider a direction.
    pub(crate) fn from_delta(dx: f64, dy: f64, threshold: f64) -> Option<Self> {
        let ax = dx.abs();
        let ay = dy.abs();
        if ax < threshold && ay < threshold {
            return None;
        }
        if ax >= ay {
            Some(if dx < 0.0 { Self::Left } else { Self::Right })
        } else {
            Some(if dy < 0.0 { Self::Up } else { Self::Down })
        }
    }
}

/// Per-slot data accumulated while tracking a multi-touch gesture on a touchscreen.
#[derive(Debug, Clone)]
pub(crate) struct TouchSlotData {
    /// Starting position when the finger first touched down.
    pub(crate) start: Point<f64, Logical>,
    /// Most recent position of this finger.
    pub(crate) current: Point<f64, Logical>,
    /// Wall-clock millisecond timestamp at first contact.
    pub(crate) start_time_ms: u64,
}

/// In-progress multi-touch gesture being accumulated for touchscreen swipe/tap detection.
#[derive(Debug, Clone, Default)]
pub(crate) struct TouchGestureBuffer {
    /// Live per-slot tracking data keyed by slot index.
    pub(crate) slots: HashMap<smithay::backend::input::TouchSlot, TouchSlotData>,
    /// Whether the gesture has already been consumed (action was fired).
    pub(crate) consumed: bool,
}

pub(crate) struct BabydraState {
    pub(crate) start_time: Instant,
    pub(crate) socket_name: OsString,
    pub(crate) display_handle: DisplayHandle,
    pub(crate) space: Space<Window>,
    pub(crate) loop_signal: LoopSignal,

    pub(crate) compositor_state: CompositorState,
    pub(crate) xdg_shell_state: XdgShellState,
    pub(crate) wlr_layer_shell_state: WlrLayerShellState,
    pub(crate) _pointer_gestures_state: PointerGesturesState,
    pub(crate) shm_state: ShmState,
    pub(crate) _output_manager_state: OutputManagerState,
    pub(crate) seat_state: SeatState<BabydraState>,
    pub(crate) data_device_state: DataDeviceState,
    pub(crate) popups: PopupManager,
    pub(crate) seat: Seat<BabydraState>,

    pub(crate) move_state: Option<MoveState>,
    pub(crate) trackpad_swipe: Option<TrackpadSwipeBuffer>,
    pub(crate) alt_tab_switcher: Option<AltTabSwitcher>,
    pub(crate) blur_engine: std::sync::Arc<std::sync::Mutex<crate::rendering::blur::BlurEngine>>,
    pub(crate) pointer_location: Point<f64, Logical>,
    pub(crate) cursor_visible: bool,
    pub(crate) pointer_pressed_buttons: std::collections::HashSet<u32>,
    pub(crate) touch_slots: std::collections::HashSet<smithay::backend::input::TouchSlot>,
    pub(crate) windows: Vec<Window>,
    /// Accumulated per-slot data for multi-touch gesture detection on touchscreens.
    pub(crate) touch_gesture: TouchGestureBuffer,
}

impl BabydraState {
    pub(crate) fn new(event_loop: &mut EventLoop<CalloopData>, display: Display<Self>) -> Self {
        let display_handle = display.handle();
        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let wlr_layer_shell_state = WlrLayerShellState::new::<Self>(&display_handle);
        let pointer_gestures_state = PointerGesturesState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        let data_device_state = DataDeviceState::new::<Self>(&display_handle);
        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(&display_handle, "babydrade");

        // The nested Winit window provides a keyboard and pointer for this first version.
        seat.add_keyboard(Default::default(), 200, 25)
            .expect("failed to initialize keyboard");
        seat.add_pointer();
        seat.add_touch();

        let socket_name = Self::init_wayland_listener(display, event_loop);

        Self {
            start_time: Instant::now(),
            socket_name,
            display_handle,
            space: Space::default(),
            loop_signal: event_loop.get_signal(),
            compositor_state,
            xdg_shell_state,
            wlr_layer_shell_state,
            _pointer_gestures_state: pointer_gestures_state,
            shm_state,
            _output_manager_state: output_manager_state,
            seat_state,
            data_device_state,
            popups: PopupManager::default(),
            seat,
            move_state: None,
            trackpad_swipe: None,
            alt_tab_switcher: None,
            blur_engine: std::sync::Arc::new(std::sync::Mutex::new(
                crate::rendering::blur::BlurEngine::new(),
            )),
            pointer_location: Point::from((100.0, 100.0)),
            cursor_visible: true,
            pointer_pressed_buttons: std::collections::HashSet::new(),
            touch_slots: std::collections::HashSet::new(),
            windows: Vec::new(),
            touch_gesture: TouchGestureBuffer::default(),
        }
    }

    fn init_wayland_listener(
        display: Display<Self>,
        event_loop: &mut EventLoop<CalloopData>,
    ) -> OsString {
        let listening_socket =
            ListeningSocketSource::new_auto().expect("failed to create Wayland socket");
        let socket_name = listening_socket.socket_name().to_os_string();
        let loop_handle = event_loop.handle();

        loop_handle
            .insert_source(listening_socket, move |client_stream, _, state| {
                state
                    .display_handle
                    .insert_client(client_stream, Arc::new(ClientState::default()))
                    .expect("failed to register Wayland client");
            })
            .expect("failed to register Wayland socket with event loop");

        loop_handle
            .insert_source(
                Generic::new(display, Interest::READ, CalloopMode::Level),
                |_, display, state| {
                    // The display remains owned by the event source for the compositor lifetime.
                    unsafe {
                        display
                            .get_mut()
                            .dispatch_clients(&mut state.state)
                            .expect("Wayland client dispatch failed");
                    }
                    Ok(PostAction::Continue)
                },
            )
            .expect("failed to register Wayland display with event loop");

        socket_name
    }
}

#[derive(Default)]
pub(crate) struct ClientState {
    pub(crate) compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}

    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}
