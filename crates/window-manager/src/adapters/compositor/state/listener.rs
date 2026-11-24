//! listener helpers.

use super::*;

impl BabydraState {
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
