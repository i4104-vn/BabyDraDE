//! Search overlay input and activation actions.

use gtk4::gio::{prelude::*, AppInfo, File as GioFile};
use smithay::utils::{Logical, Point};

use super::LauncherState;
use crate::adapters::desktop::launcher::search::index::url_encode;
use crate::adapters::desktop::launcher::search::{SearchAction, SearchText};

impl LauncherState {
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
