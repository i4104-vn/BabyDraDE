//! `InputContext` windows delegation methods.

macro_rules! windows_methods {
    () => {
        fn space(&self) -> &Space<Window> {
            &self.windows.space
        }

        fn space_mut(&mut self) -> &mut Space<Window> {
            &mut self.windows.space
        }

        fn windows(&self) -> &[Window] {
            &self.windows
        }

        fn active_window(&self) -> Option<Window> {
            BabydraState::active_window(self)
        }

        fn update_active_window(&self, window: Option<&Window>) {
            BabydraState::update_active_window(self, window);
        }

        fn stop_loop(&self) {
            self.resources.loop_signal.stop();
        }

        fn raise_window(&mut self, window: &Window) {
            BabydraState::raise_window(self, window);
        }

        fn cleanup_windows(&mut self) {
            BabydraState::cleanup_windows(self);
        }

        fn unminimize_window(&mut self, window: &Window) {
            BabydraState::unminimize_window(self, window);
        }

        fn unmaximize_window(&mut self, window: &Window) {
            BabydraState::unmaximize_window(self, window);
        }

        fn close_window(&mut self, window: &Window) {
            BabydraState::close_window(self, window);
        }

        fn minimize_window(&mut self, window: &Window) {
            BabydraState::minimize_window(self, window);
        }

        fn toggle_maximize(&mut self, window: &Window) {
            BabydraState::toggle_maximize(self, window);
        }

        fn toggle_fullscreen(&mut self, window: &Window) {
            BabydraState::toggle_fullscreen(self, window);
        }

        fn window_under(
            &self,
            point: Point<f64, Logical>,
        ) -> Option<(Window, Point<i32, Logical>)> {
            BabydraState::window_under(self, point)
        }

        fn window_border_under(
            &self,
            point: Point<f64, Logical>,
        ) -> Option<(Window, Point<i32, Logical>, ResizeEdge)> {
            BabydraState::window_border_under(self, point)
        }

        fn surface_under(
            &self,
            point: Point<f64, Logical>,
        ) -> Option<(WlSurface, Point<f64, Logical>)> {
            BabydraState::surface_under(self, point)
        }

        fn layer_under(&self, point: Point<f64, Logical>) -> Option<Layer> {
            BabydraState::layer_under(self, point)
        }

        fn titlebar_action(
            &self,
            window: &Window,
            location: Point<i32, Logical>,
            point: Point<f64, Logical>,
        ) -> Option<Action> {
            crate::adapters::compositor::state::titlebar::hit_test(window, location, point)
        }

        fn saved_window_size(&self, window: &Window) -> Option<Size<i32, Logical>> {
            // Recovering a poisoned lock keeps the last known geometry, which is
            // safer than crashing the compositor on a stale animation state.
            lock_window_state(window).saved_size
        }
    };
}
