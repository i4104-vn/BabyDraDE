use super::BabydraState;

impl BabydraState {
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
}
