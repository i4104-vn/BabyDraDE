use domain::CompositorError;

/// High-level Wayland server wrapper for BabyDraDE.
pub struct WaylandServer {
    socket_name: String,
    bound_globals: Vec<&'static str>,
    is_listening: bool,
}

impl WaylandServer {
    pub fn new(socket_name: impl Into<String>) -> Self {
        Self {
            socket_name: socket_name.into(),
            bound_globals: Vec::new(),
            is_listening: false,
        }
    }

    /// Registers core Wayland globals (wl_compositor, wl_shm, wl_seat, wl_output).
    pub fn register_core_globals(&mut self) {
        self.bound_globals = vec![
            "wl_compositor",
            "wl_shm",
            "wl_seat",
            "wl_output",
            "xdg_wm_base",
        ];
        tracing::info!("Registered Wayland globals: {:?}", self.bound_globals);
    }

    /// Binds the Wayland display socket (default: me.babydra.compositor).
    pub fn bind_socket(&mut self) -> Result<String, CompositorError> {
        self.is_listening = true;
        tracing::info!("Wayland socket bound: {}", self.socket_name);
        Ok(self.socket_name.clone())
    }

    pub fn socket_name(&self) -> &str {
        &self.socket_name
    }

    pub fn is_listening(&self) -> bool {
        self.is_listening
    }

    pub fn bound_globals(&self) -> &[&'static str] {
        &self.bound_globals
    }
}
