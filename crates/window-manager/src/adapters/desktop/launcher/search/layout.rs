//! Geometry shared by the search panel's drawing code and hit-testing.

use common::constants::launcher::{
    SEARCH_HEADER_HEIGHT, SEARCH_INPUT_HEIGHT, SEARCH_PAD_X, SEARCH_PAD_Y, SEARCH_PANEL_MAX_HEIGHT,
    SEARCH_PANEL_WIDTH, SEARCH_ROW_HEIGHT, SEARCH_SCREEN_MARGIN, SEARCH_SECTION_GAP,
};
use smithay::utils::{Logical, Point};

use super::SearchItem;

/// Logical rectangles describing the centered search panel.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SearchLayout {
    /// Outer panel (before its shadow) in logical screen coordinates.
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
    /// Query input field.
    pub(crate) input_x: f64,
    pub(crate) input_y: f64,
    pub(crate) input_w: f64,
    pub(crate) input_h: f64,
    /// Top-left of the results area and the usable row width.
    pub(crate) rows_x: f64,
    pub(crate) rows_y: f64,
    pub(crate) rows_w: f64,
}

impl SearchLayout {
    /// Centers the flat search column on the output. Individual elements fly
    /// in on their own, so the column itself never moves.
    pub(crate) fn compute(logical_w: f64, logical_h: f64, content_height: f64) -> Self {
        let margin = SEARCH_SCREEN_MARGIN;
        let width = SEARCH_PANEL_WIDTH.min(logical_w - margin * 2.0).max(160.0);
        let height = content_height
            .min(SEARCH_PANEL_MAX_HEIGHT)
            .min(logical_h - margin * 2.0)
            .max(SEARCH_INPUT_HEIGHT + SEARCH_PAD_Y * 2.0);
        let x = (logical_w - width) / 2.0;
        let y = (logical_h - height) / 2.0;

        let input_x = x + SEARCH_PAD_X;
        let input_w = width - SEARCH_PAD_X * 2.0;
        let input_y = y + SEARCH_PAD_Y;
        let rows_x = input_x;
        let rows_w = input_w;
        let rows_y = input_y + SEARCH_INPUT_HEIGHT + SEARCH_SECTION_GAP;

        Self {
            x,
            y,
            width,
            height,
            input_x,
            input_y,
            input_w,
            input_h: SEARCH_INPUT_HEIGHT,
            rows_x,
            rows_y,
            rows_w,
        }
    }

    /// Rectangle of a result row, accounting for category headings above it.
    pub(crate) fn row_rect(
        &self,
        items: &[SearchItem],
        row: usize,
    ) -> Option<(f64, f64, f64, f64)> {
        if row >= items.len() {
            return None;
        }
        let mut top = self.rows_y;
        let mut previous = None;
        for (index, item) in items.iter().enumerate().take(row + 1) {
            let category = item.category;
            if previous != Some(category) {
                top += SEARCH_HEADER_HEIGHT;
                previous = Some(category);
            }
            if index == row {
                return Some((self.rows_x, top, self.rows_w, SEARCH_ROW_HEIGHT));
            }
            top += SEARCH_ROW_HEIGHT;
        }
        None
    }

    /// Result row under a logical point, if any.
    pub(crate) fn item_at(
        &self,
        items: &[SearchItem],
        point: Point<f64, Logical>,
    ) -> Option<usize> {
        if point.x < self.rows_x || point.x > self.rows_x + self.rows_w {
            return None;
        }
        (0..items.len()).find(|&index| {
            self.row_rect(items, index).is_some_and(|(x, y, w, h)| {
                point.x >= x && point.x <= x + w && point.y >= y && point.y <= y + h
            })
        })
    }
}
