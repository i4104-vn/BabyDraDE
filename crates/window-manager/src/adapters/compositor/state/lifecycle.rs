//! lifecycle helpers.

use super::*;

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
}
