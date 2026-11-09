use std::{ffi::OsString, sync::Arc, time::Instant};

use smithay::{
    desktop::{PopupManager, Space, Window, WindowSurfaceType},
    input::{Seat, SeatState},
    reexports::{
        calloop::{
            generic::Generic, EventLoop, Interest, LoopSignal, Mode as CalloopMode, PostAction,
        },
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::wl_surface::WlSurface,
            Display, DisplayHandle,
        },
    },
    utils::{Logical, Point},
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        output::OutputManagerState,
        selection::data_device::DataDeviceState,
        shell::{wlr_layer::WlrLayerShellState, xdg::XdgShellState},
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};

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

pub(crate) struct BabydraState {
    pub(crate) start_time: Instant,
    pub(crate) socket_name: OsString,
    pub(crate) display_handle: DisplayHandle,
    pub(crate) space: Space<Window>,
    pub(crate) loop_signal: LoopSignal,

    pub(crate) compositor_state: CompositorState,
    pub(crate) xdg_shell_state: XdgShellState,
    pub(crate) wlr_layer_shell_state: WlrLayerShellState,
    pub(crate) shm_state: ShmState,
    pub(crate) _output_manager_state: OutputManagerState,
    pub(crate) seat_state: SeatState<BabydraState>,
    pub(crate) data_device_state: DataDeviceState,
    pub(crate) popups: PopupManager,
    pub(crate) seat: Seat<BabydraState>,

    pub(crate) move_state: Option<MoveState>,
    pub(crate) pointer_location: Point<f64, Logical>,
    pub(crate) pointer_pressed_buttons: std::collections::HashSet<u32>,
}

impl BabydraState {
    pub(crate) fn new(event_loop: &mut EventLoop<CalloopData>, display: Display<Self>) -> Self {
        let display_handle = display.handle();
        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let wlr_layer_shell_state = WlrLayerShellState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        let data_device_state = DataDeviceState::new::<Self>(&display_handle);
        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(&display_handle, "babydrade");

        // The nested Winit window provides a keyboard and pointer for this first version.
        seat.add_keyboard(Default::default(), 200, 25)
            .expect("failed to initialize keyboard");
        seat.add_pointer();

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
            shm_state,
            _output_manager_state: output_manager_state,
            seat_state,
            data_device_state,
            popups: PopupManager::default(),
            seat,
            move_state: None,
            pointer_location: Point::from((100.0, 100.0)),
            pointer_pressed_buttons: std::collections::HashSet::new(),
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

    pub(crate) fn window_under(
        &self,
        point: Point<f64, Logical>,
    ) -> Option<(Window, Point<i32, Logical>)> {
        // 1. First check via Smithay's element_under (handles exact input regions)
        if let Some((window, _render_loc)) = self.space.element_under(point) {
            if let Some(loc) = self.space.element_location(window) {
                return Some((window.clone(), loc));
            }
        }

        // 2. Fallback: check window bounding box and geometry in space
        for window in self.space.elements().rev() {
            if let Some(bbox) = self.space.element_bbox(window) {
                if bbox.to_f64().contains(point) {
                    if let Some(loc) = self.space.element_location(window) {
                        return Some((window.clone(), loc));
                    }
                }
            }
            if let Some(geo) = self.space.element_geometry(window) {
                if geo.to_f64().contains(point) {
                    if let Some(loc) = self.space.element_location(window) {
                        return Some((window.clone(), loc));
                    }
                }
            }
        }

        None
    }

    pub(crate) fn surface_under(
        &self,
        position: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        if let Some(ref m) = self.move_state {
            if m.has_dragged {
                return None;
            }
        }

        // 1. Check Overlay and Top layer surfaces (bars, overlays, dynamic islands)
        for output in self.space.outputs() {
            let layer_map = smithay::desktop::layer_map_for_output(output);
            for layer in layer_map.layers().rev() {
                if matches!(
                    layer.layer(),
                    smithay::wayland::shell::wlr_layer::Layer::Overlay
                        | smithay::wayland::shell::wlr_layer::Layer::Top
                ) {
                    if let Some(geo) = layer_map.layer_geometry(layer) {
                        if geo.to_f64().contains(position) {
                            if let Some((surf, surf_loc)) = layer.surface_under(
                                position - geo.loc.to_f64(),
                                WindowSurfaceType::ALL,
                            ) {
                                return Some((surf, (surf_loc + geo.loc).to_f64()));
                            }
                        }
                    }
                }
            }
        }

        // 2. Check normal desktop windows
        if let Some((window, location)) = self.space.element_under(position) {
            if let Some((surface, point)) = window
                .surface_under(position - location.to_f64(), WindowSurfaceType::ALL)
            {
                return Some((surface, (point + location).to_f64()));
            }
        }

        // 3. Check Bottom and Background layer surfaces (wallpapers, desktop widgets)
        for output in self.space.outputs() {
            let layer_map = smithay::desktop::layer_map_for_output(output);
            for layer in layer_map.layers().rev() {
                if matches!(
                    layer.layer(),
                    smithay::wayland::shell::wlr_layer::Layer::Bottom
                        | smithay::wayland::shell::wlr_layer::Layer::Background
                ) {
                    if let Some(geo) = layer_map.layer_geometry(layer) {
                        if geo.to_f64().contains(position) {
                            if let Some((surf, surf_loc)) = layer.surface_under(
                                position - geo.loc.to_f64(),
                                WindowSurfaceType::ALL,
                            ) {
                                return Some((surf, (surf_loc + geo.loc).to_f64()));
                            }
                        }
                    }
                }
            }
        }

        None
    }

    pub(crate) fn layer_under(
        &self,
        position: Point<f64, Logical>,
    ) -> Option<smithay::wayland::shell::wlr_layer::Layer> {
        for output in self.space.outputs() {
            let layer_map = smithay::desktop::layer_map_for_output(output);
            for layer in layer_map.layers().rev() {
                if let Some(geo) = layer_map.layer_geometry(layer) {
                    if geo.to_f64().contains(position) {
                        if layer
                            .surface_under(position - geo.loc.to_f64(), WindowSurfaceType::ALL)
                            .is_some()
                        {
                            return Some(layer.layer());
                        }
                    }
                }
            }
        }
        None
    }

    pub(crate) fn update_active_window(&self, window: Option<&Window>) {
        let info = if let Some(w) = window {
            if let Some(toplevel) = w.toplevel() {
                let (app_id, title) = smithay::wayland::compositor::with_states(toplevel.wl_surface(), |states| {
                    states
                        .data_map
                        .get::<smithay::wayland::shell::xdg::XdgToplevelSurfaceData>()
                        .map(|data| {
                            let guard = data.lock().unwrap();
                            (
                                guard.app_id.clone().unwrap_or_default(),
                                guard.title.clone().unwrap_or_default(),
                            )
                        })
                })
                .unwrap_or_default();
                crate::system::window::ActiveWindowInfo { app_id, title }
            } else {
                crate::system::window::ActiveWindowInfo::default()
            }
        } else {
            crate::system::window::ActiveWindowInfo::default()
        };
        crate::system::window::save_active_window(&info);
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
