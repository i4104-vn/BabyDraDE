//! `InputContext` switcher delegation methods.

macro_rules! switcher_methods {
    () => {
        fn cycle_alt_tab(&mut self, backwards: bool) {
            BabydraState::cycle_alt_tab(self, backwards);
        }

        fn begin_switcher_scrub(&mut self) -> Option<usize> {
            BabydraState::begin_switcher_scrub(self)
        }

        fn scrub_switcher(&mut self, start_x: f64, base_index: usize, current_x: f64) {
            BabydraState::scrub_switcher(self, start_x, base_index, current_x);
        }

        fn commit_switcher_selection(&mut self) {
            BabydraState::commit_switcher_selection(self);
        }

        fn switcher_is_active(&self) -> bool {
            self.overlays.alt_tab_switcher.is_some()
        }

        fn switcher_tab_pressed(&mut self, direction: i32) -> bool {
            self.overlays
                .alt_tab_switcher
                .as_mut()
                .map(|switcher| switcher.tab_pressed(direction))
                .unwrap_or(false)
        }

        fn switcher_tab_released(&mut self, direction: i32) {
            if let Some(switcher) = self.overlays.alt_tab_switcher.as_mut() {
                switcher.tab_released(direction);
            }
        }

        fn switcher_begin_close(&mut self) {
            if let Some(switcher) = self.overlays.alt_tab_switcher.as_mut() {
                switcher.begin_close();
            }
        }

        fn switcher_set_hover(&mut self, index: Option<usize>) {
            if let Some(switcher) = self.overlays.alt_tab_switcher.as_mut() {
                switcher.set_hover(index);
            }
        }

        fn switcher_drag_by(&mut self, slots: f64) {
            if let Some(switcher) = self.overlays.alt_tab_switcher.as_mut() {
                switcher.drag_by(slots);
            }
        }

        fn switcher_finish_drag(&mut self) {
            if let Some(switcher) = self.overlays.alt_tab_switcher.as_mut() {
                switcher.finish_drag();
            }
        }

        fn switcher_index_at(&self, point: (f64, f64), slide: f64) -> Option<usize> {
            let switcher = self.overlays.alt_tab_switcher.as_ref()?;
            let output = self.space.outputs().next().cloned();
            let (size, scale) = output
                .as_ref()
                .and_then(|output| {
                    let mode = output.current_mode()?;
                    let size = output.current_transform().transform_size(mode.size);
                    Some((
                        size,
                        smithay::utils::Scale::from(output.current_scale().fractional_scale()),
                    ))
                })
                .unwrap_or(((1920, 1080).into(), smithay::utils::Scale::from(1.0)));
            let logical_w = (size.w as f64 / scale.x).max(1.0);
            let logical_h = (size.h as f64 / scale.y).max(1.0);
            let layout = crate::adapters::desktop::switcher::view::compute_layout(
                switcher, logical_w, logical_h,
            )?;
            layout.panel.index_at(
                point,
                slide,
                switcher.entries.len(),
                switcher.visual_selection(),
            )
        }

        fn switcher_slide(&self, panel_width: f64) -> f64 {
            self.overlays
                .alt_tab_switcher
                .as_ref()
                .map(|switcher| (1.0 - switcher.presence().clamp(0.0, 1.0)) * panel_width)
                .unwrap_or_default()
        }

        fn switcher_window_at(&self, index: usize) -> Option<Window> {
            self.overlays
                .alt_tab_switcher
                .as_ref()
                .and_then(|switcher| switcher.entries.get(index))
                .map(|entry| entry.window.clone())
        }

        fn switcher_selected_window(&self) -> Option<Window> {
            self.overlays
                .alt_tab_switcher
                .as_ref()
                .and_then(|switcher| {
                    switcher
                        .entries
                        .get(switcher.selected_index)
                        .map(|entry| entry.window.clone())
                })
        }
    };
}
