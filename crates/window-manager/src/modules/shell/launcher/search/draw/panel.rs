//! Search panel and input row rendering.

use super::results::draw_results;
use super::*;

/// Draws the flat search column directly onto the dimmer.
pub(crate) fn draw_panel(
    context: &Context,
    state: &SearchState,
    layout: &SearchLayout,
    presence: f64,
) -> Result<(), cairo::Error> {
    if presence <= 0.001 {
        return Ok(());
    }
    draw_input(context, state, layout, presence)?;
    draw_results(context, state, layout, presence)
}

/// The query row reveals first, then settles; a thin divider separates it from
/// the results below (no filled field, no card).
fn draw_input(
    context: &Context,
    state: &SearchState,
    layout: &SearchLayout,
    presence: f64,
) -> Result<(), cairo::Error> {
    let progress = state.input_progress();
    if progress <= 0.0 {
        return Ok(());
    }
    let alpha = element_alpha(presence, progress);
    context.save()?;
    context.translate(0.0, element_dy(progress));

    set_bold_font(context, SEARCH_FONT_SIZE);
    let baseline = centered_baseline(context, layout.input_y, layout.input_h)?;
    let text_x = layout.input_x;

    if state.text().is_empty() {
        let hint = fit_text(
            context,
            &services::i18n::trans("launcher.search_placeholder"),
            layout.input_w,
        )?;
        draw_text(
            context,
            &hint,
            text_x,
            baseline,
            fade((1.0, 1.0, 1.0, 0.45), alpha),
        )?;
    } else {
        let text = state.text();
        let caret = state.caret();
        let cursor_advance = context.text_extents(&prefix(text, caret))?.x_advance();
        let total_advance = context.text_extents(text)?.x_advance();
        // Scroll only as far as needed to keep the caret on screen.
        let scroll = (cursor_advance + SEARCH_CARET_SCROLL_PAD - layout.input_w)
            .max(0.0)
            .min((total_advance + SEARCH_CARET_SCROLL_PAD - layout.input_w).max(0.0));

        let character_count = text.chars().count();
        let mut offsets = Vec::with_capacity(character_count + 1);
        for boundary in 0..=character_count {
            let advance = context.text_extents(&prefix(text, boundary))?.x_advance();
            offsets.push((text_x + advance - scroll, boundary));
        }
        state.record_caret_offsets(offsets);

        context.save()?;
        context.translate(-scroll, 0.0);

        if let Some((start, end)) = state.selection() {
            let from = text_x + context.text_extents(&prefix(text, start))?.x_advance();
            let to = text_x + context.text_extents(&prefix(text, end))?.x_advance();
            paint_rounded(
                context,
                from - 2.0,
                layout.input_y + layout.input_h * 0.16,
                to - from + 4.0,
                layout.input_h * 0.68,
                4.0,
                (1.0, 1.0, 1.0, 0.24 * alpha),
                None,
            )?;
        }

        draw_text(
            context,
            text,
            text_x,
            baseline,
            fade((1.0, 1.0, 1.0, 0.98), alpha),
        )?;

        if state.caret_visible() {
            context.set_source_rgba(1.0, 1.0, 1.0, 0.95 * alpha);
            context.rectangle(
                text_x + cursor_advance,
                layout.input_y + layout.input_h * 0.26,
                SEARCH_CARET_WIDTH,
                layout.input_h * 0.48,
            );
            context.fill()?;
        }
        context.restore()?;
    }

    let divider_y = layout.input_y + layout.input_h + 4.0;
    context.set_source_rgba(1.0, 1.0, 1.0, 0.16 * alpha);
    context.set_line_width(1.0);
    context.move_to(layout.input_x, divider_y);
    context.line_to(layout.input_x + layout.input_w, divider_y);
    context.stroke()?;

    context.restore()
}
