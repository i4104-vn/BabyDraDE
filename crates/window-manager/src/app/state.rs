//! Shared compositor state and lifecycle.
//!
//! `BabydraState` owns the live compositor components; protocol transport
//! lives in [`crate::app::bus`], request handling in [`crate::app::bridge`]
//! and window lifecycle in `crate::modules::window`.
//!
//! This module is a facade over [`crate::app::models`] and holds only the
//! cross-component orchestration functions.

use std::{ffi::OsString, sync::Arc, time::Instant};

use smithay::{
    desktop::PopupManager,
    input::pointer::CursorIcon,
    reexports::{
        calloop::{generic::Generic, EventLoop, Interest, PostAction},
        wayland_server::Display,
    },
    wayland::socket::ListeningSocketSource,
};

pub(crate) use crate::app::models::{
    BabydraState, CalloopData, ClientState, ClipboardState, ConfigSnapshot, InputState,
    OverlayState, RenderState, RuntimeResources, WaylandRuntime, WindowRegistry,
};
pub(crate) use crate::modules::window::cursor::resize_cursor_icon;
pub(crate) use crate::modules::window::ops::{is_window_exiting, start_window_opening};

// Lifecycle helpers.

impl BabydraState {
    /// Central dispatch function for all hardware and synthetic input events.
    ///
    /// # Arguments
    /// * `event` - Input event to forward to the input pipeline.
    pub(crate) fn process_input_event<I: smithay::backend::input::InputBackend>(
        &mut self,
        event: smithay::backend::input::InputEvent<I>,
    ) where
        I::Device: 'static,
    {
        crate::modules::input::process_input_event(self, event);
    }

    /// Builds the compositor state and registers its Wayland endpoints.
    ///
    /// # Arguments
    /// * `event_loop` - Loop that owns the Wayland listener and display sources.
    /// * `display` - Wayland display handed to the sources.
    /// * `load_gesture_policy` - Port used to load the cached gesture policy.
    /// * `shortcuts` - Shared shortcut service used by the input pipeline.
    /// * `registry` - Shared runtime shortcut registry.
    /// * `bus` - Sender for the compositor's local protocol peer.
    ///
    /// # Returns
    /// The fully assembled compositor state.
    pub(crate) fn new(
        event_loop: &mut EventLoop<'_, CalloopData>,
        display: Display<Self>,
        load_gesture_policy: crate::shared::ports::LoadGesturePolicy,
        shortcuts: Arc<crate::core::shortcuts::app::ShortcutService>,
        registry: Arc<crate::core::shortcuts::app::ShortcutRegistry>,
        bus: crate::protocol::BusSender,
    ) -> Self {
        let display_handle = display.handle();
        let wayland = WaylandRuntime::new(&display_handle);

        let socket_name = Self::init_wayland_listener(display, event_loop);
        let config = ConfigSnapshot::new(load_gesture_policy, shortcuts, registry);

        Self {
            bus,
            wayland,
            clipboard: ClipboardState::default(),
            config,
            input: InputState::new(),
            overlays: OverlayState::new(),
            windows: WindowRegistry::new(is_window_exiting),
            resources: RuntimeResources {
                start_time: Instant::now(),
                socket_name,
                loop_signal: event_loop.get_signal(),
                popups: PopupManager::default(),
                render: RenderState::new(),
            },
        }
    }

    /// Records that a client installed a new selection.
    ///
    /// # Arguments
    /// * `mimes` - MIME types offered by the new selection.
    pub(crate) fn note_client_selection(&mut self, mimes: Vec<String>) {
        self.clipboard.note_client_selection(mimes);
    }

    /// Requests the pending client selection and caches it on a background thread.
    pub(crate) fn update_clipboard(&mut self) {
        self.clipboard
            .update_clipboard(self.wayland.seat(), self.clipboard.sender.clone());
    }

    /// Takes ownership of an already cached clipboard selection.
    ///
    /// # Arguments
    /// * `cached_mime` - MIME type chosen for the cached selection, if any.
    pub(crate) fn complete_clipboard_capture(&mut self, cached_mime: Option<String>) {
        self.clipboard.complete_clipboard_capture(
            cached_mime,
            self.wayland.seat(),
            &self.wayland.display_handle,
        );
    }
}

// Listener helpers.

impl BabydraState {
    /// Registers the Wayland listening socket and client dispatch source.
    ///
    /// # Arguments
    /// * `display` - Wayland display owned by the registered source.
    /// * `event_loop` - Loop that owns both sources.
    ///
    /// # Returns
    /// The allocated Wayland socket name.
    pub(super) fn init_wayland_listener(
        display: Display<Self>,
        event_loop: &mut EventLoop<'_, CalloopData>,
    ) -> OsString {
        let listening_socket = ListeningSocketSource::new_auto()
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

// Settings helpers.

impl BabydraState {
    /// Reloads input device configuration from `~/.babydra/configs/input.toml`.
    ///
    /// Flushes the shared configuration cache and refreshes cached touchpad
    /// and mouse settings including natural scroll toggles.
    pub(crate) fn reload_input_config(&mut self) {
        self.config.reload();
    }
    /// Hot reloads blur and compositor configurations from disk, then notifies features.
    pub(crate) fn reload_config(&mut self) {
        self.resources.render.reload_config();
        let _ = self.bus.publish(
            crate::protocol::CONFIG_RELOAD_TOPIC,
            &crate::protocol::Event::ConfigReloaded,
        );
    }
    /// Cursor shape the renderer should show this frame.
    ///
    /// Compositor-owned drag operations and hover targets override the client's
    /// last request: borders show the matching directional resize cursor (both
    /// on hover and while dragging), the draggable titlebar shows a grab cursor,
    /// and active moves show a grabbing cursor.
    ///
    /// # Returns
    /// The cursor shape the renderer should show this frame.
    pub(crate) fn effective_cursor_icon(&self) -> CursorIcon {
        if let Some(ref resize) = self.input.resize_state {
            return resize_cursor_icon(resize.edges);
        }
        if self.input.move_state.is_some() {
            return CursorIcon::Grabbing;
        }

        let point = self.input.pointer_location;

        // Hovering a resize border shows the same arrow that a drag would use.
        if let Some((_, _, edge)) = self.window_border_under(point) {
            return resize_cursor_icon(edge);
        }

        // Hovering the draggable part of a server-side titlebar shows a grab
        // cursor. Buttons keep whatever the client requested.
        if let Some((window, location)) = self.window_under(point) {
            if matches!(
                crate::modules::window::hit_test::titlebar::hit_test(&window, location, point),
                Some(common::Action::Move)
            ) {
                return CursorIcon::Grab;
            }
        }

        self.input.cursor_icon
    }
}
