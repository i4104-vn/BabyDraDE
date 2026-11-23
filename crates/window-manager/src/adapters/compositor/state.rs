use std::{
    ffi::OsString,
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Instant,
};

use smithay::{
    desktop::{PopupManager, Space, Window},
    input::pointer::CursorIcon,
    reexports::{
        calloop::{generic::Generic, EventLoop, Interest, LoopSignal, PostAction},
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            Display, DisplayHandle, Resource,
        },
    },
    utils::{IsAlive, Logical, Point},
    wayland::{
        compositor::CompositorClientState, shell::xdg::ToplevelSurface,
        socket::ListeningSocketSource,
    },
};

use crate::adapters::window::cursor::resize_cursor_icon;
pub(crate) use crate::adapters::window::interaction::{
    MoveState, OverlayTouchState, ResizeState, TouchGestureBuffer, TrackpadSwipeBuffer,
};
use crate::adapters::window::WindowRegistry;
use crate::application::window_action::WindowAction;

mod clipboard;
mod config;
mod hit_test;
mod input;
mod input_context;
mod overlays;
mod render;
mod wayland;

pub(crate) use clipboard::ClipboardState;
pub(crate) use config::ConfigSnapshot;
pub(crate) use input::InputState;
pub(crate) use overlays::OverlayState;
pub(crate) use render::RenderState;
pub(crate) use wayland::WaylandRuntime;
pub(crate) mod launcher;
mod switcher;
pub(crate) mod titlebar;
pub(crate) mod window;

pub(crate) use window::*;

pub(crate) struct CalloopData {
    pub(crate) state: BabydraState,
    pub(crate) display_handle: DisplayHandle,
}

pub(crate) struct BabydraState {
    /// Wayland protocol globals and the compositor seat.
    pub(crate) wayland: WaylandRuntime,
    /// Clipboard capture pipeline (MIME queues owned by the component).
    pub(crate) clipboard: ClipboardState,
    /// Cached configuration snapshot owned by the input pipeline.
    pub(crate) config: ConfigSnapshot,
    /// Transient input interaction state.
    pub(crate) input: InputState,
    /// Launcher and Alt+Tab switcher state.
    pub(crate) overlays: OverlayState,
    /// Managed windows in bottom-to-top Z order.
    pub(crate) windows: WindowRegistry,
    /// Shared rendering resources.
    pub(crate) render: RenderState,

    // Flat compatibility fields below are read directly by the composition
    // root and backends, which live outside this adapter crate and cannot be
    // changed as part of this refactor.
    pub(crate) start_time: Instant,
    pub(crate) socket_name: OsString,
    pub(crate) space: Space<Window>,
    pub(crate) loop_signal: LoopSignal,
    pub(crate) popups: PopupManager,

    /// Channel used by the capture thread to hand ownership back to us.
    ///
    /// Compatibility field: the composition root installs this directly, so it
    /// stays on the facade and is forwarded to `ClipboardState` when needed.
    pub(crate) clipboard_sender:
        Option<smithay::reexports::calloop::channel::Sender<Option<String>>>,

    /// Compatibility alias of `render.blur_engine`: the backends (outside this
    /// adapter crate) read this field directly, so both names share one engine.
    pub(crate) blur_engine:
        std::sync::Arc<std::sync::Mutex<crate::adapters::rendering::blur::BlurEngine>>,
    pub(crate) pointer_location: Point<f64, Logical>,
    pub(crate) cursor_visible: bool,
}

/// Compatibility deref for callers outside this adapter crate that still use
/// the historical flat `state.launcher` / `state.alt_tab_switcher` paths. New
/// code should use `state.overlays` explicitly.
impl Deref for BabydraState {
    type Target = OverlayState;

    fn deref(&self) -> &Self::Target {
        &self.overlays
    }
}

impl DerefMut for BabydraState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.overlays
    }
}

impl BabydraState {
    /// Central dispatch function for all hardware and synthetic input events.
    pub(crate) fn process_input_event<I: smithay::backend::input::InputBackend>(
        &mut self,
        event: smithay::backend::input::InputEvent<I>,
    ) where
        I::Device: 'static,
    {
        crate::adapters::input::process_input_event(self, event);
    }

    pub(crate) fn new(
        event_loop: &mut EventLoop<'_, CalloopData>,
        display: Display<Self>,
        load_gesture_policy: crate::application::ports::LoadGesturePolicy,
        shortcuts: Arc<crate::application::shortcuts::ShortcutService>,
    ) -> Self {
        let display_handle = display.handle();
        let wayland = WaylandRuntime::new(&display_handle);

        let socket_name = Self::init_wayland_listener(display, event_loop);
        let render = RenderState::new();
        let blur_engine = render.blur_engine.clone();
        let config = ConfigSnapshot::new(load_gesture_policy, shortcuts);

        Self {
            wayland,
            clipboard: ClipboardState::default(),
            config,
            input: InputState::new(),
            overlays: OverlayState::new(),
            windows: WindowRegistry::new(is_window_exiting),
            render,
            start_time: Instant::now(),
            socket_name,
            space: Space::default(),
            loop_signal: event_loop.get_signal(),
            popups: PopupManager::default(),
            clipboard_sender: None,
            blur_engine,
            pointer_location: Point::from((100.0, 100.0)),
            cursor_visible: true,
        }
    }

    /// Records that a client installed a new selection.
    pub(crate) fn note_client_selection(&mut self, mimes: Vec<String>) {
        self.clipboard.note_client_selection(mimes);
    }

    /// Requests the pending client selection and caches it on a background thread.
    pub(crate) fn update_clipboard(&mut self) {
        self.clipboard
            .update_clipboard(&self.wayland.seat, self.clipboard_sender.clone());
    }

    /// Takes ownership of an already cached clipboard selection.
    pub(crate) fn complete_clipboard_capture(&mut self, cached_mime: Option<String>) {
        self.clipboard.complete_clipboard_capture(
            cached_mime,
            &self.wayland.seat,
            &self.wayland.display_handle,
        );
    }

    /// Removes dead windows and keeps the shared active-window state in sync.
    pub(crate) fn cleanup_windows(&mut self) {
        let before = self.windows.len();
        for window in self
            .windows
            .iter()
            .filter(|w| !w.alive())
            .cloned()
            .collect::<Vec<_>>()
        {
            crate::adapters::window::metadata::remove(&window);
        }
        self.windows.retain(|w| w.alive());

        if self.windows.len() != before || self.windows.is_empty() {
            let active = self.active_window();
            self.update_active_window(active.as_ref());
        }
    }

    /// Returns the topmost live window, excluding closing animations.
    pub(crate) fn active_window(&self) -> Option<Window> {
        self.windows.active_window(&self.space)
    }

    /// Raises a window to the top of the internal Z-order list.
    pub(crate) fn raise_window(&mut self, window: &Window) {
        self.windows.raise(window);
    }

    /// Finds a window associated with an XDG toplevel surface.
    pub(crate) fn find_window_for_surface(&self, surface: &ToplevelSurface) -> Option<Window> {
        self.windows.find_window_for_surface(surface, &self.space)
    }

    /// Persists metadata about the currently active window.
    pub(crate) fn update_active_window(&self, window: Option<&Window>) {
        let info = if let Some(window) = window {
            if let Some(toplevel) = window.toplevel() {
                let metadata = crate::adapters::window::metadata::get(window).unwrap_or_default();
                let process_id = self
                    .wayland
                    .display_handle
                    .get_client(toplevel.wl_surface().id())
                    .ok()
                    .and_then(|client| client.get_credentials(&self.wayland.display_handle).ok())
                    .and_then(|credentials| u32::try_from(credentials.pid).ok());
                services::system::window::ActiveWindowInfo {
                    app_id: metadata.app_id,
                    title: metadata.title,
                    process_id,
                }
            } else {
                services::system::window::ActiveWindowInfo::default()
            }
        } else {
            services::system::window::ActiveWindowInfo::default()
        };
        services::system::window::save_active_window(&info);
    }

    /// Handles system-wide window commands (e.g. from topbar, dock, gestures).
    pub(crate) fn handle_window_action(&mut self, action: WindowAction) {
        let active = self.active_window();
        let Some(window) = active else {
            return;
        };
        match action {
            WindowAction::Minimize => self.minimize_window(&window),
            WindowAction::Maximize => self.toggle_maximize(&window),
            WindowAction::Fullscreen => self.toggle_fullscreen(&window),
            WindowAction::Close => self.close_window(&window),
        }
    }

    /// Reloads input device configuration from `~/.babydra/configs/input.toml`.
    ///
    /// Flushes the shared configuration cache and refreshes cached touchpad
    /// and mouse settings including natural scroll toggles.
    pub(crate) fn reload_input_config(&mut self) {
        self.config.reload();
    }

    /// Hot reloads blur and compositor configurations from disk.
    pub(crate) fn reload_config(&mut self) {
        self.render.reload_config();
    }

    /// Cursor shape the renderer should show this frame.
    ///
    /// Compositor-owned drag operations and hover targets override the client's
    /// last request: borders show the matching directional resize cursor (both
    /// on hover and while dragging), the draggable titlebar shows a grab cursor,
    /// and active moves show a grabbing cursor.
    pub(crate) fn effective_cursor_icon(&self) -> CursorIcon {
        if let Some(ref resize) = self.input.resize_state {
            return resize_cursor_icon(resize.edges);
        }
        if self.input.move_state.is_some() {
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

        self.input.cursor_icon
    }

    fn init_wayland_listener(
        display: Display<Self>,
        event_loop: &mut EventLoop<'_, CalloopData>,
    ) -> OsString {
        let listening_socket =
            ListeningSocketSource::new_auto()
                .expect("failed to create the Wayland listening socket (check XDG_RUNTIME_DIR)");
        let socket_name = listening_socket.socket_name().to_os_string();
        let loop_handle = event_loop.handle();

        loop_handle
            .insert_source(listening_socket, move |client_stream, _, state| {
                // A single bad client must not take down the whole compositor.
                if let Err(error) = state
                    .state
                    .wayland
                    .display_handle
                    .insert_client(client_stream, Arc::new(ClientState::default()))
                {
                    tracing::error!("failed to register Wayland client: {error}");
                }
            })
            .expect("failed to register the Wayland listening socket with the calloop event loop");

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
                    let dispatch = unsafe { display.get_mut().dispatch_clients(&mut state.state) };
                    if let Err(error) = dispatch {
                        // Protocol errors from one client are logged and the loop keeps
                        // running; panicking here would kill every other client.
                        tracing::error!("Wayland client dispatch failed: {error}");
                    }
                    Ok(PostAction::Continue)
                },
            )
            .expect("failed to register the Wayland display with the calloop event loop");

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
