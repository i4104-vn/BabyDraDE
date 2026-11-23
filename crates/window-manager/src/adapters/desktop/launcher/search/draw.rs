//! Cairo drawing for the centered search panel. Every colour is multiplied by
//! the overlay presence so the fly-in and the return to the dock share one
//! continuous, jank-free fade.

use cairo::{Context, FontSlant, FontWeight};
use common::constants::launcher::{
    HAIRLINE, PANEL_SURFACE, SEARCH_CARET_WIDTH, SEARCH_FONT_SIZE, SEARCH_HEADER_FONT_SIZE,
    SEARCH_HEADER_HEIGHT, SEARCH_ICON_SIZE, SEARCH_PAD_X, SEARCH_ROW_HEIGHT,
};
use gtk4::gdk::prelude::GdkCairoContextExt;
use ui_kit::drawing::{draw_text, fit_text, paint_rounded, rounded_rect, set_bold_font};

use super::layout::SearchLayout;
use super::state::SearchState;
use super::{SearchCategory, SearchItem};

type Color = (f64, f64, f64, f64);

fn fade(color: Color, presence: f64) -> Color {
    (color.0, color.1, color.2, color.3 * presence)
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

/// Draws an application icon or a small shape-based badge for files and web.
fn draw_badge(
    context: &Context,
    item: &SearchItem,
    center_x: f64,
    center_y: f64,
    presence: f64,
) -> Result<(), cairo::Error> {
    let size = SEARCH_ICON_SIZE;
    let x = center_x - size / 2.0;
    let y = center_y - size / 2.0;

    if let Some(ref pixbuf) = item.icon {
        let (pw, ph) = (pixbuf.width() as f64, pixbuf.height() as f64);
        if pw > 0.0 && ph > 0.0 {
            context.save()?;
            rounded_rect(context, x, y, size, size, size * 0.24);
            context.clip();
            let scale = size / pw.max(ph);
            context.translate(x + (size - pw * scale) / 2.0, y + (size - ph * scale) / 2.0);
            context.scale(scale, scale);
            context.set_source_pixbuf(pixbuf, 0.0, 0.0);
            context.paint_with_alpha(presence)?;
            context.restore()?;
            return Ok(());
        }
    }

    match item.category {
        SearchCategory::Apps => {
            paint_rounded(
                context,
                x,
                y,
                size,
                size,
                size * 0.24,
                fade((0.36, 0.52, 0.92, 0.95), presence),
                Some(fade(HAIRLINE, presence)),
            )?;
            let initial = item
                .label
                .chars()
                .next()
                .unwrap_or('A')
                .to_uppercase()
                .to_string();
            set_bold_font(context, size * 0.5);
            let extents = context.text_extents(&initial)?;
            draw_text(
                context,
                &initial,
                center_x - extents.x_advance() / 2.0 - extents.x_bearing(),
                centered_baseline(context, y, size)?,
                fade((1.0, 1.0, 1.0, 0.95), presence),
            )
        }
        SearchCategory::Files => {
            if item.is_dir {
                // Folder: a tab plus a wider body.
                paint_rounded(
                    context,
                    x + 1.0,
                    y + 4.0,
                    size * 0.48,
                    size * 0.26,
                    3.0,
                    fade((0.93, 0.70, 0.28, 0.95), presence),
                    None,
                )?;
                paint_rounded(
                    context,
                    x,
                    y + size * 0.26,
                    size,
                    size * 0.62,
                    4.0,
                    fade((0.98, 0.78, 0.36, 0.95), presence),
                    Some(fade(HAIRLINE, presence)),
                )
            } else {
                // File: a page with a folded corner.
                paint_rounded(
                    context,
                    x + 3.0,
                    y,
                    size - 7.0,
                    size,
                    4.0,
                    fade((0.62, 0.70, 0.82, 0.95), presence),
                    Some(fade(HAIRLINE, presence)),
                )?;
                context.move_to(x + size - 9.0, y + 1.0);
                context.line_to(x + size - 3.0, y + 7.0);
                context.line_to(x + size - 9.0, y + 7.0);
                context.close_path();
                context.set_source_rgba(1.0, 1.0, 1.0, 0.45 * presence);
                context.fill()
            }
        }
        SearchCategory::Web => {
            paint_rounded(
                context,
                x,
                y,
                size,
                size,
                size / 2.0,
                fade((0.20, 0.60, 0.52, 0.95), presence),
                Some(fade(HAIRLINE, presence)),
            )?;
            let mid = center_y;
            context.set_source_rgba(1.0, 1.0, 1.0, 0.75 * presence);
            context.set_line_width(1.2);
            context.new_path();
            context.arc(center_x, mid, size * 0.34, 0.0, std::f64::consts::TAU);
            context.stroke()?;
            context.move_to(center_x - size * 0.34, mid);
            context.line_to(center_x + size * 0.34, mid);
            context.stroke()?;
            context.save()?;
            context.translate(center_x, mid);
            context.scale(0.44, 1.0);
            context.new_path();
            context.arc(0.0, 0.0, size * 0.34, 0.0, std::f64::consts::TAU);
            context.stroke()?;
            context.restore()
        }
    }
}

/// Draws the whole search panel into the translated, scaled Cairo context.
pub(crate) fn draw_panel(
    context: &Context,
    state: &SearchState,
    layout: &SearchLayout,
    presence: f64,
) -> Result<(), cairo::Error> {
    if presence <= 0.001 {
        return Ok(());
    }

    // Soft, layered shadow.
    for (grow, alpha) in [(18.0, 0.05), (10.0, 0.08), (4.0, 0.12)] {
        paint_rounded(
            context,
            layout.x - grow,
            layout.y - grow + 4.0,
            layout.width + grow * 2.0,
            layout.height + grow * 2.0,
            common::constants::launcher::SEARCH_PANEL_RADIUS + grow,
            (0.0, 0.0, 0.0, alpha * presence),
            None,
        )?;
    }

    paint_rounded(
        context,
        layout.x,
        layout.y,
        layout.width,
        layout.height,
        common::constants::launcher::SEARCH_PANEL_RADIUS,
        fade(PANEL_SURFACE, presence),
        Some(fade(HAIRLINE, presence)),
    )?;

    draw_input(context, state, layout, presence)?;
    draw_results(context, state, layout, presence)
}

fn draw_input(
    context: &Context,
    state: &SearchState,
    layout: &SearchLayout,
    presence: f64,
) -> Result<(), cairo::Error> {
    let radius = layout.input_h / 2.0;
    paint_rounded(
        context,
        layout.input_x,
        layout.input_y,
        layout.input_w,
        layout.input_h,
        radius,
        fade((1.0, 1.0, 1.0, 0.07), presence),
        Some(fade(HAIRLINE, presence)),
    )?;

    set_bold_font(context, SEARCH_FONT_SIZE);
    let baseline = centered_baseline(context, layout.input_y, layout.input_h)?;
    let text_x = layout.input_x + SEARCH_PAD_X;
    let max_width = layout.input_w - SEARCH_PAD_X * 2.0 - SEARCH_CARET_WIDTH - 4.0;

    if state.query().is_empty() {
        let hint = fit_text(
            context,
            &services::i18n::trans("launcher.search_placeholder"),
            max_width,
        )?;
        draw_text(
            context,
            &hint,
            text_x,
            baseline,
            fade((1.0, 1.0, 1.0, 0.42), presence),
        )?;
    } else {
        let text = fit_text(context, state.query(), max_width)?;
        draw_text(
            context,
            &text,
            text_x,
            baseline,
            fade((1.0, 1.0, 1.0, 0.96), presence),
        )?;
        let advance = context.text_extents(&text)?.x_advance();
        context.set_source_rgba(1.0, 1.0, 1.0, 0.85 * presence);
        context.rectangle(
            text_x + advance + 3.0,
            layout.input_y + layout.input_h * 0.28,
            SEARCH_CARET_WIDTH,
            layout.input_h * 0.44,
        );
        context.fill()?;
    }
    Ok(())
}

fn draw_results(
    context: &Context,
    state: &SearchState,
    layout: &SearchLayout,
    presence: f64,
) -> Result<(), cairo::Error> {
    let mut top = layout.rows_y;
    let mut previous: Option<SearchCategory> = None;

    for (index, item) in state.items().iter().enumerate() {
        if previous != Some(item.category) {
            set_bold_font(context, SEARCH_HEADER_FONT_SIZE);
            draw_text(
                context,
                &item.category.title(),
                layout.rows_x + 2.0,
                top + SEARCH_HEADER_HEIGHT * 0.72,
                fade((1.0, 1.0, 1.0, 0.45), presence),
            )?;
            top += SEARCH_HEADER_HEIGHT;
            previous = Some(item.category);
        }

        if index == state.selected() {
            paint_rounded(
                context,
                layout.rows_x - 6.0,
                top + 2.0,
                layout.rows_w + 12.0,
                SEARCH_ROW_HEIGHT - 4.0,
                10.0,
                fade((1.0, 1.0, 1.0, 0.11), presence),
                Some(fade(HAIRLINE, presence)),
            )?;
        }

        draw_badge(
            context,
            item,
            layout.rows_x + SEARCH_ICON_SIZE / 2.0 + 6.0,
            top + SEARCH_ROW_HEIGHT / 2.0,
            presence,
        )?;

        let text_x = layout.rows_x + SEARCH_ICON_SIZE + 20.0;
        let max_width = layout.rows_w - SEARCH_ICON_SIZE - 28.0;

        set_bold_font(context, SEARCH_FONT_SIZE);
        let label = fit_text(context, &item.label, max_width)?;
        draw_text(
            context,
            &label,
            text_x,
            top + SEARCH_ROW_HEIGHT * 0.40,
            fade((1.0, 1.0, 1.0, 0.94), presence),
        )?;

        if !item.detail.is_empty() {
            set_regular_font(context, 10.5);
            let detail = fit_text(context, &item.detail, max_width)?;
            draw_text(
                context,
                &detail,
                text_x,
                top + SEARCH_ROW_HEIGHT * 0.74,
                fade((1.0, 1.0, 1.0, 0.44), presence),
            )?;
        }

        top += SEARCH_ROW_HEIGHT;
    }

    if state.items().is_empty() {
        set_regular_font(context, SEARCH_FONT_SIZE);
        let message = services::i18n::trans("launcher.no_results");
        let extents = context.text_extents(&message)?;
        draw_text(
            context,
            &message,
            layout.rows_x + (layout.rows_w - extents.x_advance()) / 2.0,
            layout.rows_y + SEARCH_HEADER_HEIGHT,
            fade((1.0, 1.0, 1.0, 0.55), presence),
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cairo::ImageSurface;

    #[test]
    fn draws_a_panel_without_error() {
        let surface = ImageSurface::create(cairo::Format::ARgb32, 800, 600).expect("surface");
        let context = Context::new(&surface).expect("context");
        let mut state = SearchState::new();
        state.push_char('a', &[]);
        let layout = SearchLayout::compute(800.0, 600.0, state.content_height(), 0.0);
        assert!(draw_panel(&context, &state, &layout, 1.0).is_ok());
    }
}
