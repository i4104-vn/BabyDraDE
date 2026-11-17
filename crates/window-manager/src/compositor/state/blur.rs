use super::BabydraState;

impl BabydraState {
    /// Hot reloads blur and compositor configurations from disk
    pub(crate) fn reload_config(&mut self) {
        if let Ok(mut engine) = self.blur_engine.lock() {
            engine.reload_config();
            tracing::info!(
                "Win+F5: Configuration reloaded from disk ({} app rules registered for blur)",
                engine.params.app_rules.len()
            );
        }
    }
}
