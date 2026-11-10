use super::BabydraState;

impl BabydraState {
    /// Add an application to the blur engine rules
    #[allow(dead_code)]
    pub(crate) fn apply_blur_to_app(&mut self, app_id: &str) {
        if let Ok(mut engine) = self.blur_engine.lock() {
            engine.add_app(app_id);
        }
    }

    /// Remove an application from the blur engine rules
    #[allow(dead_code)]
    pub(crate) fn remove_blur_from_app(&mut self, app_id: &str) {
        if let Ok(mut engine) = self.blur_engine.lock() {
            engine.remove_app(app_id);
        }
    }

    /// Toggle blur on the currently active window
    pub(crate) fn toggle_blur_active_window(&mut self) -> Option<(String, bool)> {
        let active_window = self.space.elements().next_back()?.clone();
        let app_id = {
            let engine = self.blur_engine.lock().ok()?;
            engine
                .get_window_app_id(&active_window)
                .or_else(|| engine.get_window_title(&active_window))
                .unwrap_or_else(|| "active-window".to_string())
        };

        let new_state = {
            let mut engine = self.blur_engine.lock().ok()?;
            engine.toggle_app(&app_id)
        };

        Some((app_id, new_state))
    }

    /// Enable or disable blur for all windows globally
    #[allow(dead_code)]
    pub(crate) fn set_blur_all(&mut self, blur_all: bool) {
        if let Ok(mut engine) = self.blur_engine.lock() {
            engine.set_blur_all(blur_all);
        }
    }
}
