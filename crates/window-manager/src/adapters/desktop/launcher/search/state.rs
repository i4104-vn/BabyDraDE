//! Mutable state for the launcher's search overlay: query, results, selection
//! and the two time-driven tweens that make the panel fly in and fade out.

use animation::{Easing, Tween, FADE_OUT_DURATION, SEARCH_FLY_DURATION, SEARCH_OPEN_DURATION};
use common::constants::launcher::{
    SEARCH_FLY_DISTANCE, SEARCH_HEADER_HEIGHT, SEARCH_INPUT_HEIGHT, SEARCH_PAD_Y,
    SEARCH_ROW_HEIGHT, SEARCH_SECTION_GAP,
};
use smithay::utils::{Logical, Point};

use super::super::LauncherApp;
use super::index::{build_results, FileIndex};
use super::layout::SearchLayout;
use super::{SearchAction, SearchItem, SearchPanelKey};

/// Search state owned by the compositor launcher.
pub(crate) struct SearchState {
    query: String,
    items: Vec<SearchItem>,
    selected: usize,
    /// Bumped whenever the pixels of the panel need redrawing.
    revision: u64,
    /// Opacity of the dimmer and the panel (also suppresses the docked capsule).
    open: Tween,
    /// 0 = panel below its resting place, 1 = centered.
    fly: Tween,
    /// One-shot, bounded filesystem snapshot used by the file category.
    files: FileIndex,
    /// Geometry from the previous render, reused for pointer hit-testing.
    layout: Option<SearchLayout>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchState {
    pub(crate) fn new() -> Self {
        Self {
            query: String::new(),
            items: Vec::new(),
            selected: 0,
            revision: 0,
            open: Tween::settled(0.0),
            fly: Tween::settled(0.0),
            files: FileIndex::new(),
            layout: None,
        }
    }

    /// True while the search surface owns keyboard input, including the short
    /// fade-out window after the last character is erased.
    pub(crate) fn is_active(&self) -> bool {
        !self.query.is_empty() || self.open.value() > 0.01
    }

    /// Current dimmer/panel opacity in `0..=1`.
    pub(crate) fn presence(&self) -> f64 {
        self.open.value().clamp(0.0, 1.0)
    }

    /// Vertical distance added to the centered Y position (fly-in from below).
    pub(crate) fn fly_offset(&self) -> f64 {
        (1.0 - self.fly.value()) * SEARCH_FLY_DISTANCE
    }

    pub(crate) fn query(&self) -> &str {
        &self.query
    }

    pub(crate) fn items(&self) -> &[SearchItem] {
        &self.items
    }

    pub(crate) fn selected(&self) -> usize {
        self.selected
    }

    /// Max of the animating tweens' frame ticks, or `u64::MAX` when at rest.
    /// Quantized to the shared 120 fps budget by [`Tween::frame_tick`].
    pub(crate) fn animation_tick(&self) -> u64 {
        let ticks = [
            self.open.is_animating().then(|| self.open.frame_tick()),
            self.fly.is_animating().then(|| self.fly.frame_tick()),
        ];
        ticks.into_iter().flatten().max().unwrap_or(u64::MAX)
    }

    pub(crate) fn panel_key(&self, region: super::SearchRegion) -> SearchPanelKey {
        (self.revision, self.animation_tick(), region)
    }

    /// Height the content asks for before the layout clamps it to the output.
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

    pub(crate) fn set_layout(&mut self, layout: SearchLayout) {
        self.layout = Some(layout);
    }

    /// Result row under a logical point, using the last rendered geometry.
    pub(crate) fn item_at(&self, point: Point<f64, Logical>) -> Option<usize> {
        self.layout?.item_at(&self.items, point)
    }

    /// Whether a logical point falls on the panel (including its input field).
    pub(crate) fn panel_contains(&self, point: Point<f64, Logical>) -> bool {
        self.layout.is_some_and(|layout| {
            point.x >= layout.x
                && point.x <= layout.x + layout.width
                && point.y >= layout.y
                && point.y <= layout.y + layout.height
        })
    }

    /// Inserts a typed character and refreshes all three result sections.
    pub(crate) fn push_char(&mut self, character: char, apps: &[LauncherApp]) {
        self.query.push(character);
        self.refresh(apps);
        self.reveal();
    }

    /// Removes the last character; an empty query dismisses the search surface.
    pub(crate) fn backspace(&mut self, apps: &[LauncherApp]) {
        if self.query.pop().is_none() {
            return;
        }
        if self.query.is_empty() {
            self.dismiss();
        } else {
            self.refresh(apps);
        }
        self.revision = self.revision.wrapping_add(1);
    }

    /// Escape / explicit clear: fly back to the docked launcher.
    pub(crate) fn clear(&mut self) {
        if self.query.is_empty() && self.open.target() == 0.0 {
            return;
        }
        self.query.clear();
        self.dismiss();
        self.revision = self.revision.wrapping_add(1);
    }

    /// Hard reset used when the launcher itself opens or finishes closing.
    pub(crate) fn reset(&mut self) {
        self.query.clear();
        self.items.clear();
        self.selected = 0;
        self.open = Tween::settled(0.0);
        self.fly = Tween::settled(0.0);
        self.layout = None;
        self.revision = self.revision.wrapping_add(1);
    }

    /// Moves the highlight across the flattened result list.
    pub(crate) fn move_selection(&mut self, delta: i32) {
        if self.items.is_empty() {
            return;
        }
        let count = self.items.len() as i32;
        let next = (self.selected as i32 + delta).rem_euclid(count) as usize;
        if next != self.selected {
            self.selected = next;
            self.revision = self.revision.wrapping_add(1);
        }
    }

    /// Selects a row directly (pointer input).
    pub(crate) fn select(&mut self, index: usize) {
        if index < self.items.len() && self.selected != index {
            self.selected = index;
            self.revision = self.revision.wrapping_add(1);
        }
    }

    /// Action attached to the highlighted row, if any.
    pub(crate) fn activate(&self) -> Option<SearchAction> {
        self.items
            .get(self.selected)
            .map(|item| item.action.clone())
    }

    fn refresh(&mut self, apps: &[LauncherApp]) {
        self.items = build_results(&self.query, apps, &self.files);
        self.selected = self.selected.min(self.items.len().saturating_sub(1));
        self.revision = self.revision.wrapping_add(1);
    }

    fn reveal(&mut self) {
        if self.open.target() != 1.0 {
            self.open
                .retarget(1.0, SEARCH_OPEN_DURATION, Easing::EaseOutCubic);
            self.fly
                .retarget(1.0, SEARCH_FLY_DURATION, Easing::EaseOutBack);
        }
    }

    fn dismiss(&mut self) {
        if self.open.target() != 0.0 {
            self.open
                .retarget(0.0, FADE_OUT_DURATION, Easing::EaseOutCubic);
            self.fly
                .retarget(0.0, FADE_OUT_DURATION, Easing::EaseInOutCubic);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_character_activates_search() {
        let mut state = SearchState::new();
        assert!(!state.is_active());
        state.push_char('a', &[]);
        assert!(state.is_active());
        assert_eq!(state.query(), "a");
        // The web fallback is always the last row, whatever the local index finds.
        assert!(!state.items().is_empty());
        assert_eq!(
            state.items().last().unwrap().category,
            super::super::SearchCategory::Web
        );
    }

    #[test]
    fn erasing_the_last_character_starts_the_return_transition() {
        let mut state = SearchState::new();
        state.push_char('a', &[]);
        state.backspace(&[]);
        assert!(state.query().is_empty());
        assert_eq!(state.open.target(), 0.0);
    }

    #[test]
    fn selection_wraps_across_rows() {
        let mut state = SearchState::new();
        state.push_char('a', &[]);
        let last = state.items().len() - 1;
        state.move_selection(-1);
        assert_eq!(state.selected(), last);
        state.move_selection(1);
        assert_eq!(state.selected(), 0);
    }
}
