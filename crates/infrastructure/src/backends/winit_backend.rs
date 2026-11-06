use domain::{CompositorBackend, CompositorError, Output, OutputId, Rect};

/// Winit backend implementation for development mode (runs inside a window).
#[derive(Debug, Default)]
pub struct WinitBackend {
    initialized: bool,
    outputs: Vec<Output>,
}

impl WinitBackend {
    pub fn new() -> Self {
        Self {
            initialized: false,
            outputs: Vec::new(),
        }
    }
}

impl CompositorBackend for WinitBackend {
    fn name(&self) -> &str {
        "winit"
    }

    fn is_drm(&self) -> bool {
        false
    }

    fn initialize(&mut self) -> Result<(), CompositorError> {
        self.initialized = true;
        // Default virtual output for dev
        self.outputs = vec![Output::new(
            OutputId::new(1),
            "Winit-Output-1",
            Rect::new(0, 0, 1920, 1080),
            1.0,
        )];
        tracing::info!("Winit development backend initialized");
        Ok(())
    }

    fn outputs(&self) -> Vec<Output> {
        self.outputs.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_winit_backend() {
        let mut backend = WinitBackend::new();
        assert_eq!(backend.name(), "winit");
        assert!(!backend.is_drm());
        assert!(backend.initialize().is_ok());
        assert_eq!(backend.outputs().len(), 1);
    }
}
