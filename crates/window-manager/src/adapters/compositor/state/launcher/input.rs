//! Launcher input, selection and search actions.

use animation::NAVIGATION_DURATION;
use smithay::utils::{Logical, Point};

use super::LauncherState;

impl LauncherState {
    pub(crate) fn is_visible(&self) -> bool {
        self.visible && !self.dock.is_closing()
    }

    pub(crate) fn toggle(&mut self) -> bool {
        if self.visible && !self.dock.is_closing() {
            self.close();
        } else {
            let selected = self
                .dock
                .selected
                .unwrap_or(0)
                .min(self.apps.len().saturating_sub(1));
            self.visible = true;
            self.dock.open(selected, true);
            self.cache_key = None;
            // A fresh opening always starts from the plain docked capsule.
            self.search.reset();
            self.search_panel_key = None;
            self.search_backdrop_key = None;
        }
        self.visible
    }

    pub(crate) fn close(&mut self) {
        if !self.visible {
            return;
        }
        self.dock.begin_close();
        self.search.clear();
        self.cache_key = None;
    }

    pub(super) fn finish_close(&mut self) {
        self.visible = false;
        self.dock.finish_close();
        self.search.reset();
        self.cache_key = None;
        self.backdrop_cache_key = None;
        self.search_panel_key = None;
        self.search_backdrop_key = None;
    }

    pub(crate) fn arrow_pressed(&mut self, direction: i32) -> bool {
        self.visible && !self.search.is_active() && self.dock.key_pressed(direction)
    }

    pub(crate) fn arrow_released(&mut self, direction: i32) {
        self.dock.key_released(direction);
    }

    pub(crate) fn move_selection(&mut self, delta: i32) {
        if self.search.is_active() {
            return;
        }
        self.dock
            .move_selection(self.apps.len(), delta, NAVIGATION_DURATION);
        self.cache_key = None;
    }

    pub(crate) fn activate(&mut self) {
        if self.search.is_active() {
            self.search_activate();
            return;
        }
        if !self.dock.is_closing() {
            if let Some(index) = self.dock.display_selected() {
                self.launch(index);
            }
        }
    }

    pub(crate) fn scroll_by(&mut self, slots: f64) {
        if self.search.is_active() {
            return;
        }
        self.dock.scroll_by(self.apps.len(), slots);
        self.cache_key = None;
    }

    pub(crate) fn drag_by(&mut self, slots: f64) {
        if self.search.is_active() {
            return;
        }
        self.dock.drag_by(self.apps.len(), slots);
        self.cache_key = None;
    }

    pub(crate) fn finish_drag(&mut self) {
        if self.search.is_active() {
            return;
        }
        self.dock.finish_drag(self.apps.len());
        self.cache_key = None;
    }

    pub(crate) fn index_at(&self, point: Point<f64, Logical>) -> Option<usize> {
        if !self.visible || self.dock.is_closing() || self.search.is_active() {
            return None;
        }
        self.panel().index_at(
            (point.x, point.y),
            self.slide_offset(),
            self.apps.len(),
            self.dock.position(),
        )
    }

    pub(crate) fn is_over_panel(&self, point: Point<f64, Logical>) -> bool {
        self.visible
            && !self.dock.is_closing()
            && !self.search.is_active()
            && self
                .panel()
                .contains((point.x, point.y), self.slide_offset())
    }

    pub(crate) fn update_hover(&mut self, point: Point<f64, Logical>) -> bool {
        if self.search.is_active() {
            return false;
        }
        let Some(index) = self.index_at(point) else {
            if self.dock.hovered.is_some() {
                self.dock.clear_hover();
                return true;
            }
            return false;
        };
        if self.dock.hovered == Some(index) {
            return false;
        }
        // Hover changes only the highlight; it must not retarget `position`,
        // otherwise the carousel moves and hit-testing starts to chase itself.
        self.dock.set_hover(index);
        true
    }
}
