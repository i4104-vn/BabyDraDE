use domain::{CompositorBackend, CompositorError, Output};

/// Direct Rendering Manager (DRM/KMS) backend skeleton for production bare-metal execution.
#[derive(Debug, Default)]
pub struct DrmBackend {
    initialized: bool,
    outputs: Vec<Output>,
}

impl DrmBackend {
    pub fn new() -> Self {
        Self {
            initialized: false,
            outputs: Vec::new(),
        }
    }
}

impl CompositorBackend for DrmBackend {
    fn name(&self) -> &str {
        "drm"
    }

    fn is_drm(&self) -> bool {
        true
    }

    fn initialize(&mut self) -> Result<(), CompositorError> {
        if cfg!(windows) {
            return Err(CompositorError::BackendInitializationFailed(
                "DRM backend is only supported on Linux bare-metal TTY".into(),
            ));
        }

        self.initialized = true;
        tracing::info!("DRM/KMS bare-metal backend initialized");
        Ok(())
    }

    fn outputs(&self) -> Vec<Output> {
        self.outputs.clone()
    }
}
