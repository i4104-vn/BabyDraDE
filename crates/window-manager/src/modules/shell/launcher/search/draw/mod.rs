//! Cairo drawing for the flat, card-less search column.
//!
//! There is no panel surface: the content sits directly on the full-screen
//! dimmer. The query input and every result element each own a slice of one
//! master timeline, so they fade and fly in one after another from top to
//! bottom instead of appearing as a single block.

use animation::Easing;
use cairo::{Context, FontSlant, FontWeight};
use common::constants::launcher::{
    HAIRLINE, SEARCH_CARET_SCROLL_PAD, SEARCH_CARET_WIDTH, SEARCH_ELEMENT_FLY_DISTANCE,
    SEARCH_FONT_SIZE, SEARCH_HEADER_FONT_SIZE, SEARCH_HEADER_HEIGHT, SEARCH_ICON_SIZE,
    SEARCH_ROW_HEIGHT, SEARCH_SELECTION_BAR,
};
use gtk4::gdk::prelude::GdkCairoContextExt;
use ui_kit::drawing::{draw_text, fit_text, paint_rounded, rounded_rect, set_bold_font};

use super::layout::SearchLayout;
use super::state::SearchState;
use super::{SearchCategory, SearchItem};

type Color = (f64, f64, f64, f64);

fn fade(color: Color, alpha: f64) -> Color {
    (color.0, color.1, color.2, color.3 * alpha)
}

fn set_regular_font(context: &Context, size: f64) {
    context.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
    context.set_font_size(size);
}

/// Baseline that vertically centers a line of text within `[top, top+height]`.
fn centered_baseline(context: &Context, top: f64, height: f64) -> Result<f64, cairo::Error> {
    let font = context.font_extents()?;
    Ok(top + (height - (font.ascent() + font.descent())) / 2.0 + font.ascent())
}

fn element_alpha(presence: f64, progress: f64) -> f64 {
    presence * Easing::EaseOutCubic.apply(progress)
}

fn element_dy(progress: f64) -> f64 {
    (1.0 - Easing::EaseOutBack.apply(progress)) * SEARCH_ELEMENT_FLY_DISTANCE
}

/// GTK `TextIter` offsets count characters, not UTF-8 bytes.
fn prefix(text: &str, characters: usize) -> String {
    text.chars().take(characters).collect()
}

pub(crate) use panel::draw_panel;

#[cfg(test)]
mod tests {
    use super::*;
    use cairo::ImageSurface;

    #[test]
    fn draws_a_flat_panel_without_error() {
        let surface = ImageSurface::create(cairo::Format::ARgb32, 800, 600).expect("surface");
        let context = Context::new(&surface).expect("context");
        let mut state = SearchState::new();
        state.text_mut().insert("a");
        state.sync(&[]);
        let layout = SearchLayout::compute(800.0, 600.0, state.content_height());
        assert!(draw_panel(&context, &state, &layout, 1.0).is_ok());
    }
}

mod badge;

mod panel;

mod results;
