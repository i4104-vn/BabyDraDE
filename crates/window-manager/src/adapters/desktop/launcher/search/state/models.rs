//! Search overlay state model and its basic accessors.

use std::cell::RefCell;

use animation::{Easing, Stagger};
use common::constants::launcher::{
    SEARCH_HEADER_HEIGHT, SEARCH_INPUT_HEIGHT, SEARCH_INPUT_SPAN, SEARCH_PAD_Y, SEARCH_ROW_HEIGHT,
    SEARCH_ROW_SPAN, SEARCH_ROW_START, SEARCH_ROW_STEP, SEARCH_SECTION_GAP,
};

use super::super::index::FileIndex;
use super::super::layout::SearchLayout;
use super::super::SearchItem;
use super::super::SearchText;
use super::handler::caret_phase;

/// Minimal state shared by input, rendering and hit-testing.
pub(crate) struct SearchState {
    /// Query text and caret; input code edits this directly.
    pub(super) text: SearchText,
    /// Query the current `items` were built from. Compared each frame.
    pub(super) query: String,
    pub(super) items: Vec<SearchItem>,
    /// Highlighted result row.
    pub(super) selected: usize,
    /// Single clock for the dimmer and the staggered input/result reveal.
    pub(super) reveal: Stagger,
    /// Bounded one-shot filesystem snapshot.
    pub(super) files: FileIndex,
    /// Geometry from the last render, reused for hit-testing.
    pub(super) layout: Option<SearchLayout>,
    /// Caret X positions (character offsets) recorded while drawing.
    pub(super) caret_offsets: RefCell<Vec<(f64, usize)>>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchState {
    /// Creates an empty overlay state with a fresh file index.
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

    /// Returns the current query string.
    pub(crate) fn text(&self) -> &str {
        self.text.text()
    }

    /// Returns the current caret character offset.
    pub(crate) fn caret(&self) -> usize {
        self.text.caret()
    }

    /// Returns the current character selection, if any.
    pub(crate) fn selection(&self) -> Option<(usize, usize)> {
        self.text.selection()
    }

    /// Returns whether the overlay has query text or is still fading out.
    pub(crate) fn is_active(&self) -> bool {
        !self.text.is_empty() || self.reveal.value() > 0.01
    }

    /// Dimmer opacity, derived from the same clock as the stagger.
    pub(crate) fn presence(&self) -> f64 {
        Easing::EaseOutCubic.apply(self.reveal.value().clamp(0.0, 1.0))
    }

    /// Returns the reveal progress of the query row.
    pub(crate) fn input_progress(&self) -> f64 {
        self.reveal.progress(0)
    }

    /// Returns the reveal progress of one result row.
    pub(crate) fn row_progress(&self, index: usize) -> f64 {
        self.reveal.progress(index + 1)
    }

    /// Returns the animation tick used by the render cache.
    pub(crate) fn animation_tick(&self) -> u64 {
        self.reveal.tween().cache_tick()
    }

    /// Returns whether the caret is in its visible blink phase.
    pub(crate) fn caret_visible(&self) -> bool {
        caret_phase().is_multiple_of(2)
    }

    /// Returns the built result rows.
    pub(crate) fn items(&self) -> &[SearchItem] {
        &self.items
    }

    /// Returns the highlighted result row index.
    pub(crate) fn selected(&self) -> usize {
        self.selected
    }

    /// Clears the query text without resetting the overlay animation.
    pub(crate) fn clear(&mut self) {
        self.text.clear();
    }

    /// Resets the query, results, animation and cached geometry.
    pub(crate) fn reset(&mut self) {
        self.text.clear();
        self.query.clear();
        self.items.clear();
        self.selected = 0;
        self.reveal.snap_to(0.0);
        self.layout = None;
        self.caret_offsets.borrow_mut().clear();
    }

    /// Computes the panel height needed for the current result sections.
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

    /// Places the caret after a pointer click in the query row.
    pub(crate) fn set_caret_index(&mut self, index: usize) {
        self.text.set_caret_index(index);
    }
}
