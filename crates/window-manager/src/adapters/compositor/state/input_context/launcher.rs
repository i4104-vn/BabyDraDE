//! `InputContext` launcher delegation methods.

macro_rules! launcher_methods {
    () => {
        fn launcher_is_visible(&self) -> bool {
            self.overlays.launcher.is_visible()
        }

        fn launcher_arrow_pressed(&mut self, direction: i32) -> bool {
            self.overlays.launcher.arrow_pressed(direction)
        }

        fn launcher_move_selection(&mut self, direction: i32) {
            self.overlays.launcher.move_selection(direction);
        }

        fn launcher_arrow_released(&mut self, direction: i32) {
            self.overlays.launcher.arrow_released(direction);
        }

        fn launcher_activate(&mut self) {
            self.overlays.launcher.activate();
        }

        fn launcher_close(&mut self) {
            self.overlays.launcher.close();
        }

        fn launcher_toggle(&mut self) {
            self.overlays.launcher.toggle();
        }

        fn launcher_index_at(&self, position: Point<f64, Logical>) -> Option<usize> {
            self.overlays.launcher.index_at(position)
        }

        fn launcher_launch(&mut self, index: usize) {
            self.overlays.launcher.launch(index);
        }

        fn launcher_update_hover(&mut self, position: Point<f64, Logical>) {
            self.overlays.launcher.update_hover(position);
        }

        fn launcher_drag_by(&mut self, slots: f64) {
            self.overlays.launcher.drag_by(slots);
        }

        fn launcher_finish_drag(&mut self) {
            self.overlays.launcher.finish_drag();
        }

        fn launcher_is_over_panel(&self, position: Point<f64, Logical>) -> bool {
            self.overlays.launcher.is_over_panel(position)
        }

        fn launcher_scroll_by(&mut self, slots: f64) {
            self.overlays.launcher.scroll_by(slots);
        }

        fn launcher_search_is_active(&self) -> bool {
            self.overlays.launcher.search_is_active()
        }

        fn launcher_search_text_mut(&mut self) -> &mut SearchText {
            self.overlays.launcher.search_text_mut()
        }

        fn launcher_search_move(&mut self, delta: i32) {
            self.overlays.launcher.search_move(delta);
        }

        fn launcher_search_escape(&mut self) -> bool {
            self.overlays.launcher.search_escape()
        }

        fn launcher_search_click(&mut self, position: Point<f64, Logical>) -> bool {
            self.overlays.launcher.search_click(position)
        }
    };
}
