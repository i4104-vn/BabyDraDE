//! Overlay port implementation for `BabydraState`.

use smithay::{
    desktop::Window,
    utils::{Logical, Point, Scale},
};

use crate::shared::ports::{OverlayBridge, QueryText};
use crate::modules::shell::switcher::render::compute_layout;
use crate::app::models::runtime::BabydraState;

impl OverlayBridge for BabydraState {
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

    fn launcher_search_text_mut(&mut self) -> &mut dyn QueryText {
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
        let output = self.windows.space.outputs().next().cloned();
        let (size, scale) = output
            .as_ref()
            .and_then(|output| {
                let mode = output.current_mode()?;
                let size = output.current_transform().transform_size(mode.size);
                Some((size, Scale::from(output.current_scale().fractional_scale())))
            })
            .unwrap_or(((1920, 1080).into(), Scale::from(1.0)));
        let logical_w = (size.w as f64 / scale.x).max(1.0);
        let logical_h = (size.h as f64 / scale.y).max(1.0);
        let layout = compute_layout(switcher, logical_w, logical_h)?;
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
}
