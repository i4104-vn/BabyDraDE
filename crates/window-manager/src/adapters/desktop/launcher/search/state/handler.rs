//! Search result, animation and hit-testing logic.

use std::time::Instant;

use animation::{FADE_OUT_DURATION, SEARCH_REVEAL_DURATION};
use common::constants::launcher::SEARCH_CARET_BLINK_MS;
use smithay::utils::{Logical, Point};

use super::super::index::build_results;
use super::super::layout::SearchLayout;
use super::super::{SearchAction, SearchPanelKey, SearchRegion};
use super::models::SearchState;
use crate::adapters::desktop::launcher::LauncherApp;

/// Blink phase from a process-wide clock, so no per-instance timestamp is kept.
pub(super) fn caret_phase() -> u64 {
    static EPOCH: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    let epoch = *EPOCH.get_or_init(Instant::now);
    (epoch.elapsed().as_millis() / SEARCH_CARET_BLINK_MS as u128) as u64
}

impl SearchState {
    /// Rebuilds results only when the query actually changed. Called once per
    /// frame by the renderer; no signals or edit commands are involved.
    pub(crate) fn sync(&mut self, apps: &[LauncherApp]) {
        let query = self.text().to_string();
        if query == self.query {
            return;
        }
        let was_empty = self.query.is_empty();
        self.query = query;
        self.items = build_results(&self.query, apps, &self.files);
        self.selected = self.selected.min(self.items.len().saturating_sub(1));

        if self.query.is_empty() != was_empty {
            if self.query.is_empty() {
                self.reveal.dismiss(FADE_OUT_DURATION);
            } else {
                self.reveal.reveal(SEARCH_REVEAL_DURATION);
            }
        }
    }

    /// Builds the cache key for a redraw of the given physical region.
    pub(crate) fn panel_key(&self, region: SearchRegion) -> SearchPanelKey {
        (
            self.query.clone(),
            self.caret(),
            self.selection(),
            self.selected,
            self.animation_tick(),
            caret_phase(),
            region,
        )
    }

    /// Moves the highlighted row by `delta`, wrapping around the result list.
    pub(crate) fn move_selection(&mut self, delta: i32) {
        if self.items.is_empty() {
            return;
        }
        let count = self.items.len() as i32;
        self.selected = (self.selected as i32 + delta).rem_euclid(count) as usize;
    }

    /// Highlights a specific result row when it exists.
    pub(crate) fn select(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected = index;
        }
    }

    /// Returns the action attached to the highlighted result row.
    pub(crate) fn activate(&self) -> Option<SearchAction> {
        self.items
            .get(self.selected)
            .map(|item| item.action.clone())
    }

    /// Stores the panel geometry recorded by the last render.
    pub(crate) fn set_layout(&mut self, layout: SearchLayout) {
        self.layout = Some(layout);
    }

    /// Returns the result row under `point`, if any.
    pub(crate) fn item_at(&self, point: Point<f64, Logical>) -> Option<usize> {
        self.layout?.item_at(&self.items, point)
    }

    /// Returns the query input rectangle recorded by the last render.
    pub(crate) fn input_rect(&self) -> Option<(f64, f64, f64, f64)> {
        self.layout.map(|layout| {
            (
                layout.input_x,
                layout.input_y,
                layout.input_w,
                layout.input_h,
            )
        })
    }

    /// Records caret X positions while drawing, for pointer hit-testing.
    pub(crate) fn record_caret_offsets(&self, offsets: Vec<(f64, usize)>) {
        *self.caret_offsets.borrow_mut() = offsets;
    }

    /// Returns the closest recorded caret offset to screen X `x`.
    pub(crate) fn caret_at_x(&self, x: f64) -> Option<usize> {
        self.caret_offsets
            .borrow()
            .iter()
            .min_by(|(a, _), (b, _)| (a - x).abs().total_cmp(&(b - x).abs()))
            .map(|(_, offset)| *offset)
    }

    /// Returns whether `point` lies inside the recorded panel rectangle.
    pub(crate) fn panel_contains(&self, point: Point<f64, Logical>) -> bool {
        self.layout.is_some_and(|layout| {
            point.x >= layout.x
                && point.x <= layout.x + layout.width
                && point.y >= layout.y
                && point.y <= layout.y + layout.height
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_is_the_only_thing_that_rebuilds_results() {
        let mut state = SearchState::new();
        state.text_mut().insert("a");
        state.sync(&[]);
        assert!(state.is_active());
        assert_eq!(state.query, "a");
        assert!(!state.items().is_empty());
        assert_eq!(
            state.items().last().unwrap().category,
            super::super::super::SearchCategory::Web
        );

        // Caret movement alone must not touch the result set.
        state.text_mut().move_caret(-1, false, false);
        state.sync(&[]);
        assert_eq!(state.query, "a");
        assert_eq!(state.caret(), 0);
    }

    #[test]
    fn empty_query_dismisses() {
        let mut state = SearchState::new();
        state.text_mut().insert("a");
        state.sync(&[]);
        assert_eq!(state.reveal.target(), 1.0);

        state.text_mut().backspace();
        state.sync(&[]);
        assert!(state.query.is_empty());
        assert_eq!(state.reveal.target(), 0.0);
    }
}
