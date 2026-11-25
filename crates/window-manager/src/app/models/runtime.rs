//! Compositor runtime models and the facade state structure.

// Compositor state models.


pub(crate) use client::ClientState;
pub(crate) use runtime::RuntimeResources;
pub(crate) use state::{BabydraState, CalloopData};

// Inlined from core/state/models/client.rs.
mod client {
    //! Wayland client data attached to each connected client.

    use smithay::{
        reexports::wayland_server::backend::{ClientData, ClientId, DisconnectReason},
        wayland::compositor::CompositorClientState,
    };

    #[derive(Default)]
    pub(crate) struct ClientState {
        pub(crate) compositor_state: CompositorClientState,
    }

    impl ClientData for ClientState {
        fn initialized(&self, _client_id: ClientId) {}

        fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
    }
}

// Inlined from core/state/models/runtime.rs.
mod runtime {
    //! Shared runtime resources owned by the compositor state.

    use std::{ffi::OsString, time::Instant};

    use smithay::{desktop::PopupManager, reexports::calloop::LoopSignal};

    use crate::app::state::RenderState;

    /// Long-lived resources shared by the event loop, backends and renderers.
    pub(crate) struct RuntimeResources {
        pub(crate) start_time: Instant,
        pub(crate) socket_name: OsString,
        pub(crate) loop_signal: LoopSignal,
        pub(crate) popups: PopupManager,
        pub(crate) render: RenderState,
    }
}

// Inlined from core/state/models/state.rs.
mod state {
    //! Compositor state data models.

    use smithay::reexports::wayland_server::DisplayHandle;

    use crate::modules::window::WindowRegistry;

    use crate::{
        app::state::{InputState, OverlayState},
        modules::wayland::{clipboard::ClipboardState, runtime::WaylandRuntime},
        shared::config::snapshot::ConfigSnapshot,
    };

    use super::RuntimeResources;

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
        /// Managed window space and runtime resources.
        pub(crate) resources: RuntimeResources,
    }
}
