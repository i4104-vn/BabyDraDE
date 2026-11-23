//! Rendering resources owned by the compositor state.

use std::sync::{Arc, Mutex};

use crate::adapters::rendering::blur::BlurEngine;

/// Owns the shared blur pipeline used by every frame renderer.
pub(crate) struct RenderState {
    pub(crate) blur_engine: Arc<Mutex<BlurEngine>>,
}

impl RenderState {
    pub(crate) fn new() -> Self {
        Self {
            blur_engine: Arc::new(Mutex::new(BlurEngine::new())),
        }
    }

    /// Hot reloads blur configuration from disk.
    pub(crate) fn reload_config(&self) {
        if let Ok(mut engine) = self.blur_engine.lock() {
            engine.reload_config();
            tracing::info!(
                "Win+F5: Configuration reloaded from disk ({} app rules registered for blur)",
                engine.params.app_rules.len()
            );
        }
    }
}
