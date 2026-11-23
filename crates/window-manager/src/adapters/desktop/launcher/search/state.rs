//! State for the launcher's search overlay.
//!
//! The [`SearchText`] model is the source of truth for the query. Everything
//! else is derived once per frame: results come from the query string, the
//! dimmer and stagger come from one timeline, and the cache key is built from
//! the values that are actually read while drawing.

use std::{cell::RefCell, time::Instant};

use animation::{Easing, Stagger, FADE_OUT_DURATION, SEARCH_REVEAL_DURATION};
use common::constants::launcher::{
    SEARCH_CARET_BLINK_MS, SEARCH_HEADER_HEIGHT, SEARCH_INPUT_HEIGHT, SEARCH_INPUT_SPAN,
    SEARCH_PAD_Y, SEARCH_ROW_HEIGHT, SEARCH_ROW_SPAN, SEARCH_ROW_START, SEARCH_ROW_STEP,
    SEARCH_SECTION_GAP,
};
use smithay::utils::{Logical, Point};

use super::super::LauncherApp;
use super::index::{build_results, FileIndex};
use super::layout::SearchLayout;
use super::{SearchAction, SearchItem, SearchPanelKey, SearchText};

/// Blink phase from a process-wide clock, so no per-instance timestamp is kept.
fn caret_phase() -> u64 {
    static EPOCH: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    let epoch = *EPOCH.get_or_init(Instant::now);
    (epoch.elapsed().as_millis() / SEARCH_CARET_BLINK_MS as u128) as u64
}

/// Minimal state shared by input, rendering and hit-testing.
pub(crate) struct SearchState {
    /// Query text and caret; input code edits this directly.
    text: SearchText,
    /// Query the current `items` were built from. Compared each frame.
    query: String,
    items: Vec<SearchItem>,
    /// Highlighted result row.
    selected: usize,
    /// Single clock for the dimmer and the staggered input/result reveal.
    reveal: Stagger,
    /// Bounded one-shot filesystem snapshot.
    files: FileIndex,
    /// Geometry from the last render, reused for hit-testing.
    layout: Option<SearchLayout>,
    /// Caret X positions (character offsets) recorded while drawing.
    caret_offsets: RefCell<Vec<(f64, usize)>>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchState {
    pub(crate) fn new() -> Self {
        Self {
            text: SearchText::new(),
            query: String::new(),
            items: Vec::new(),
            selected: 0,
            reveal: Stagger::new(
                SEARCH_INPUT_SPAN,
                SEARCH_ROW_START,
                SEARCH_ROW_SPAN,
                SEARCH_ROW_STEP,
            ),
            files: FileIndex::new(),
            layout: None,
            caret_offsets: RefCell::new(Vec::new()),
        }
    }

    /// Mutable text model, used by the input adapter.
    pub(crate) fn text_mut(&mut self) -> &mut SearchText {
        &mut self.text
    }

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

    pub(crate) fn is_active(&self) -> bool {
        !self.text.is_empty() || self.reveal.value() > 0.01
    }

    pub(crate) fn text(&self) -> &str {
        self.text.text()
    }

    pub(crate) fn caret(&self) -> usize {
        self.text.caret()
    }

    pub(crate) fn selection(&self) -> Option<(usize, usize)> {
        self.text.selection()
    }

    /// Dimmer opacity, derived from the same clock as the stagger.
    pub(crate) fn presence(&self) -> f64 {
        Easing::EaseOutCubic.apply(self.reveal.value().clamp(0.0, 1.0))
    }

    pub(crate) fn input_progress(&self) -> f64 {
        self.reveal.progress(0)
    }

    pub(crate) fn row_progress(&self, index: usize) -> f64 {
        self.reveal.progress(index + 1)
    }

    pub(crate) fn animation_tick(&self) -> u64 {
        self.reveal.tween().cache_tick()
    }

    pub(crate) fn caret_visible(&self) -> bool {
        caret_phase().is_multiple_of(2)
    }

    pub(crate) fn panel_key(&self, region: super::SearchRegion) -> SearchPanelKey {
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

    pub(crate) fn items(&self) -> &[SearchItem] {
        &self.items
    }

    pub(crate) fn selected(&self) -> usize {
        self.selected
    }

    pub(crate) fn clear(&mut self) {
        self.text.clear();
    }

    pub(crate) fn reset(&mut self) {
        self.text.clear();
        self.query.clear();
        self.items.clear();
        self.selected = 0;
        self.reveal.snap_to(0.0);
        self.layout = None;
        self.caret_offsets.borrow_mut().clear();
    }

    pub(crate) fn move_selection(&mut self, delta: i32) {
        if self.items.is_empty() {
            return;
        }
        let count = self.items.len() as i32;
        self.selected = (self.selected as i32 + delta).rem_euclid(count) as usize;
    }

    pub(crate) fn select(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected = index;
        }
    }

    pub(crate) fn activate(&self) -> Option<SearchAction> {
        self.items
            .get(self.selected)
            .map(|item| item.action.clone())
    }

    pub(crate) fn content_height(&self) -> f64 {
        let mut height = SEARCH_PAD_Y * 2.0 + SEARCH_INPUT_HEIGHT + SEARCH_SECTION_GAP;
        let mut previous = None;
        for item in &self.items {
            if previous != Some(item.category) {
                height += SEARCH_HEADER_HEIGHT;
                previous = Some(item.category);
            }
            height += SEARCH_ROW_HEIGHT;
        }
        height
    }

    // -- geometry / hit-testing -------------------------------------------

    pub(crate) fn set_layout(&mut self, layout: SearchLayout) {
        self.layout = Some(layout);
    }

    pub(crate) fn item_at(&self, point: Point<f64, Logical>) -> Option<usize> {
        self.layout?.item_at(&self.items, point)
    }

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

    pub(crate) fn record_caret_offsets(&self, offsets: Vec<(f64, usize)>) {
        *self.caret_offsets.borrow_mut() = offsets;
    }

    pub(crate) fn caret_at_x(&self, x: f64) -> Option<usize> {
        self.caret_offsets
            .borrow()
            .iter()
            .min_by(|(a, _), (b, _)| (a - x).abs().total_cmp(&(b - x).abs()))
            .map(|(_, offset)| *offset)
    }

    /// Places the caret after a pointer click in the query row.
    pub(crate) fn set_caret_index(&mut self, index: usize) {
        self.text.set_caret_index(index);
    }

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
            super::super::SearchCategory::Web
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
