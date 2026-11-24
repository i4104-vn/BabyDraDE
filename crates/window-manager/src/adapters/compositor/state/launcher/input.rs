//! Launcher input, selection and search actions.

use animation::NAVIGATION_DURATION;
use gtk4::gio::{prelude::*, AppInfo, File as GioFile};
use smithay::utils::{Logical, Point};

use super::LauncherState;
use crate::adapters::desktop::launcher::search::index::url_encode;
use crate::adapters::desktop::launcher::search::{SearchAction, SearchText};

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

    // -- search overlay ----------------------------------------------------

    pub(crate) fn search_is_active(&self) -> bool {
        self.visible && self.search.is_active()
    }

    /// Mutable query model backing the search input.
    pub(crate) fn search_text_mut(&mut self) -> &mut SearchText {
        self.search.text_mut()
    }

    pub(crate) fn search_move(&mut self, delta: i32) {
        if self.search_is_active() {
            self.search.move_selection(delta);
        }
    }

    /// Escape returns to the docked launcher; a second Escape closes it.
    pub(crate) fn search_escape(&mut self) -> bool {
        if !self.search_is_active() {
            return false;
        }
        self.search.clear();
        true
    }

    pub(crate) fn search_activate(&mut self) {
        let Some(action) = self.search.activate() else {
            return;
        };
        match action {
            SearchAction::App(index) => self.launch(index),
            SearchAction::Path(path) => {
                let uri = GioFile::for_path(&path).uri().to_string();
                let _ = AppInfo::launch_default_for_uri(&uri, None::<&gtk4::gio::AppLaunchContext>);
                self.close();
            }
            SearchAction::Web(query) => {
                let url = format!("https://www.google.com/search?q={}", url_encode(&query));
                let _ = AppInfo::launch_default_for_uri(&url, None::<&gtk4::gio::AppLaunchContext>);
                self.close();
            }
        }
    }

    /// Handles a click while the search surface owns the screen. Returns true
    /// when the click was consumed (result launched, or dismissed back to dock).
    pub(crate) fn search_click(&mut self, point: Point<f64, Logical>) -> bool {
        if !self.search_is_active() {
            return false;
        }
        if let Some((x, y, width, height)) = self.search.input_rect() {
            if point.x >= x && point.x <= x + width && point.y >= y && point.y <= y + height {
                if let Some(offset) = self.search.caret_at_x(point.x) {
                    self.search.set_caret_index(offset);
                }
                return true;
            }
        }
        if let Some(index) = self.search.item_at(point) {
            self.search.select(index);
            self.search_activate();
        } else if !self.search.panel_contains(point) {
            self.search.clear();
        }
        true
    }

    pub(crate) fn launch(&mut self, index: usize) {
        if let Some(app) = self.apps.get(index) {
            let _ = app.info.launch(&[], None::<&gtk4::gio::AppLaunchContext>);
        }
        self.close();
    }
}
