use std::{collections::HashMap, ffi::OsString, sync::Arc, time::Instant};

use smithay::{
    desktop::{PopupManager, Space, Window},
    input::{pointer::CursorIcon, Seat, SeatState},
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
    reexports::{
        calloop::{generic::Generic, EventLoop, Interest, LoopSignal, PostAction},
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            Display, DisplayHandle,
        },
    },
    utils::{Logical, Point},
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        cursor_shape::CursorShapeManagerState,
        output::OutputManagerState,
        pointer_gestures::PointerGesturesState,
        selection::data_device::DataDeviceState,
        shell::{
            wlr_layer::WlrLayerShellState,
            xdg::{decoration::XdgDecorationState, XdgShellState},
        },
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};

use crate::domain::cursor::resize_cursor_icon;

mod active_window;
mod blur;
mod clipboard;
mod hit_test;
pub(crate) mod launcher;
mod switcher;
pub(crate) mod titlebar;
pub(crate) mod window;

use crate::adapters::desktop::switcher::AltTabSwitcher;
pub(crate) use window::*;

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
pub(crate) struct ResizeState {
    pub(crate) window: Window,
    pub(crate) initial_window_location: Point<i32, Logical>,
    pub(crate) initial_window_size: smithay::utils::Size<i32, Logical>,
    pub(crate) initial_pointer_location: Point<f64, Logical>,
    pub(crate) edges: ResizeEdge,
    pub(crate) button: u32,
}

impl ResizeState {
    pub(crate) fn resizes_left(&self) -> bool {
        matches!(
            self.edges,
            ResizeEdge::Left | ResizeEdge::TopLeft | ResizeEdge::BottomLeft
        )
    }
    pub(crate) fn resizes_right(&self) -> bool {
        matches!(
            self.edges,
            ResizeEdge::Right | ResizeEdge::TopRight | ResizeEdge::BottomRight
        )
    }
    pub(crate) fn resizes_top(&self) -> bool {
        matches!(
            self.edges,
            ResizeEdge::Top | ResizeEdge::TopLeft | ResizeEdge::TopRight
        )
    }
    pub(crate) fn resizes_bottom(&self) -> bool {
        matches!(
            self.edges,
            ResizeEdge::Bottom | ResizeEdge::BottomLeft | ResizeEdge::BottomRight
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TrackpadSwipeBuffer {
    pub(crate) fingers: u32,
    pub(crate) time: u32,
    pub(crate) total_delta: Point<f64, Logical>,
    pub(crate) updates: Vec<(Point<f64, Logical>, u32)>,
    /// 3-finger horizontal drag is scrubbing the switcher selection.
    pub(crate) scrubbing: bool,
    pub(crate) scrub_start_x: f64,
    pub(crate) scrub_base_index: usize,
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
    /// 3-finger horizontal drag is scrubbing the switcher selection.
    pub(crate) scrubbing: bool,
    pub(crate) scrub_start_x: f64,
    pub(crate) scrub_base_index: usize,
}

pub(crate) struct BabydraState {
    pub(crate) start_time: Instant,
    pub(crate) socket_name: OsString,
    pub(crate) display_handle: DisplayHandle,
    pub(crate) space: Space<Window>,
    pub(crate) loop_signal: LoopSignal,

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
    pub(crate) popups: PopupManager,
    pub(crate) seat: Seat<BabydraState>,

    /// MIME types of a client selection that still needs to be cached.
    pub(crate) clipboard_pending: Option<Vec<String>>,
    /// MIME types of a capture currently running in a background thread.
    pub(crate) clipboard_capture: Option<Vec<String>>,
    /// Channel used by the capture thread to hand ownership back to us.
    pub(crate) clipboard_sender:
        Option<smithay::reexports::calloop::channel::Sender<Option<String>>>,

    pub(crate) move_state: Option<MoveState>,
    pub(crate) resize_state: Option<ResizeState>,
    pub(crate) last_titlebar_click: Option<(std::time::Instant, Point<f64, Logical>, Window)>,
    pub(crate) trackpad_swipe: Option<TrackpadSwipeBuffer>,
    pub(crate) alt_tab_switcher: Option<AltTabSwitcher>,
    /// Compositor-owned application launcher (shown via its shortcut).
    pub(crate) launcher: launcher::LauncherState,
    pub(crate) blur_engine:
        std::sync::Arc<std::sync::Mutex<crate::adapters::rendering::blur::BlurEngine>>,
    pub(crate) pointer_location: Point<f64, Logical>,
    pub(crate) cursor_visible: bool,
    /// Last cursor shape requested by the focused Wayland client.
    pub(crate) cursor_icon: CursorIcon,
    pub(crate) pointer_pressed_buttons: std::collections::HashSet<u32>,
    pub(crate) touch_slots: std::collections::HashSet<smithay::backend::input::TouchSlot>,
    pub(crate) windows: Vec<Window>,
    /// Active single-finger touch interaction with compositor overlays (launcher or switcher).
    pub(crate) overlay_touch: Option<crate::adapters::input::touch::events::OverlayTouchState>,
    /// Accumulated per-slot data for multi-touch gesture detection on touchscreens.
    pub(crate) touch_gesture: TouchGestureBuffer,

    /// Cached touchpad configuration loaded from `~/.babydra/configs/input.toml`.
    /// Updated when a device is added/configured; avoids per-event disk reads.
    pub(crate) touchpad_natural_scroll: bool,
    /// Cached mouse/pointer configuration natural scroll setting.
    pub(crate) mouse_natural_scroll: bool,
    /// Framework-free gesture policy snapshot, refreshed only on config reload.
    pub(crate) gesture_config: crate::application::ports::GesturePolicy,
    load_gesture_policy: crate::application::ports::LoadGesturePolicy,
}

impl BabydraState {
    pub(crate) fn new(
        event_loop: &mut EventLoop<'_, CalloopData>,
        display: Display<Self>,
        load_gesture_policy: crate::application::ports::LoadGesturePolicy,
    ) -> Self {
        let display_handle = display.handle();
        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let xdg_decoration_state = XdgDecorationState::new::<Self>(&display_handle);
        let wlr_layer_shell_state = WlrLayerShellState::new::<Self>(&display_handle);
        let pointer_gestures_state = PointerGesturesState::new::<Self>(&display_handle);
        let cursor_shape_state = CursorShapeManagerState::new::<Self>(&display_handle);
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
            _xdg_decoration_state: xdg_decoration_state,
            wlr_layer_shell_state,
            _pointer_gestures_state: pointer_gestures_state,
            _cursor_shape_state: cursor_shape_state,
            shm_state,
            _output_manager_state: output_manager_state,
            seat_state,
            data_device_state,
            popups: PopupManager::default(),
            seat,
            clipboard_pending: None,
            clipboard_capture: None,
            clipboard_sender: None,
            move_state: None,
            resize_state: None,
            last_titlebar_click: None,
            trackpad_swipe: None,
            alt_tab_switcher: None,
            launcher: launcher::LauncherState::new(),
            blur_engine: std::sync::Arc::new(std::sync::Mutex::new(
                crate::adapters::rendering::blur::BlurEngine::new(),
            )),
            pointer_location: Point::from((100.0, 100.0)),
            cursor_visible: true,
            cursor_icon: CursorIcon::Default,
            pointer_pressed_buttons: std::collections::HashSet::new(),
            touch_slots: std::collections::HashSet::new(),
            windows: Vec::new(),
            overlay_touch: None,
            touch_gesture: TouchGestureBuffer::default(),
            touchpad_natural_scroll: services::config::load_touchpad_config().natural_scroll,
            mouse_natural_scroll: services::config::load_mouse_config().natural_scroll,
            gesture_config: load_gesture_policy(),
            load_gesture_policy,
        }
    }

    /// Reloads input device configuration from `~/.babydra/configs/input.toml`.
    ///
    /// Flushes the shared configuration cache and refreshes cached touchpad
    /// and mouse settings including natural scroll toggles.
    pub(crate) fn reload_input_config(&mut self) {
        services::config::invalidate_cache();
        self.touchpad_natural_scroll = services::config::load_touchpad_config().natural_scroll;
        self.mouse_natural_scroll = services::config::load_mouse_config().natural_scroll;
        self.gesture_config = (self.load_gesture_policy)();
    }

    /// Cursor shape the renderer should show this frame.
    ///
    /// Compositor-owned drag operations and hover targets override the client's
    /// last request: borders show the matching directional resize cursor (both
    /// on hover and while dragging), the draggable titlebar shows a grab cursor,
    /// and active moves show a grabbing cursor.
    pub(crate) fn effective_cursor_icon(&self) -> CursorIcon {
        if let Some(ref resize) = self.resize_state {
            return resize_cursor_icon(resize.edges);
        }
        if self.move_state.is_some() {
            return CursorIcon::Grabbing;
        }

        let point = self.pointer_location;

        // Hovering a resize border shows the same arrow that a drag would use.
        if let Some((_, _, edge)) = self.window_border_under(point) {
            return resize_cursor_icon(edge);
        }

        // Hovering the draggable part of a server-side titlebar shows a grab
        // cursor. Buttons keep whatever the client requested.
        if let Some((window, location)) = self.window_under(point) {
            if matches!(
                crate::adapters::compositor::state::titlebar::hit_test(&window, location, point),
                Some(common::Action::Move)
            ) {
                return CursorIcon::Grab;
            }
        }

        self.cursor_icon
    }

    fn init_wayland_listener(
        display: Display<Self>,
        event_loop: &mut EventLoop<'_, CalloopData>,
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
                Generic::new(
                    display,
                    Interest::READ,
                    smithay::reexports::calloop::Mode::Level,
                ),
                |_, display, state| {
                    // The display remains owned by the event source for the compositor lifetime.
                    // SAFETY: Pointer and lifetime invariants were verified at this call site.
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
