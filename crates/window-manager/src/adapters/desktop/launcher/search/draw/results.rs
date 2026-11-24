//! Search result rows and headings rendering.

use super::badge::draw_badge;
use super::*;

/// Headings and result rows share one queue: each takes its own delayed slice
/// of the master timeline and flies up independently.
pub(super) fn draw_results(
    context: &Context,
    state: &SearchState,
    layout: &SearchLayout,
    presence: f64,
) -> Result<(), cairo::Error> {
    let mut top = layout.rows_y;
    let mut previous: Option<SearchCategory> = None;
    let mut element = 0usize;

    for (index, item) in state.items().iter().enumerate() {
        if previous != Some(item.category) {
            let progress = state.row_progress(element);
            if progress > 0.0 {
                let alpha = element_alpha(presence, progress);
                context.save()?;
                context.translate(0.0, element_dy(progress));
                set_bold_font(context, SEARCH_HEADER_FONT_SIZE);
                draw_text(
                    context,
                    &item.category.title(),
                    layout.rows_x + 2.0,
                    top + SEARCH_HEADER_HEIGHT * 0.72,
                    fade((1.0, 1.0, 1.0, 0.5), alpha),
                )?;
                context.restore()?;
            }
            element += 1;
            top += SEARCH_HEADER_HEIGHT;
            previous = Some(item.category);
        }

        let progress = state.row_progress(element);
        if progress > 0.0 {
            let alpha = element_alpha(presence, progress);
            context.save()?;
            context.translate(0.0, element_dy(progress));

            // Flat selection: just a short accent bar on the left edge.
            if index == state.selected() {
                context.set_source_rgba(1.0, 1.0, 1.0, 0.9 * alpha);
                context.rectangle(
                    layout.rows_x - 14.0,
                    top + 8.0,
                    SEARCH_SELECTION_BAR,
                    SEARCH_ROW_HEIGHT - 16.0,
                );
                context.fill()?;
            }

            draw_badge(
                context,
                item,
                layout.rows_x + SEARCH_ICON_SIZE / 2.0,
                top + SEARCH_ROW_HEIGHT / 2.0,
                alpha,
            )?;

            let text_x = layout.rows_x + SEARCH_ICON_SIZE + 20.0;
            let max_width = layout.rows_w - SEARCH_ICON_SIZE - 28.0;

            set_bold_font(context, SEARCH_FONT_SIZE);
            let label = fit_text(context, &item.label, max_width)?;
            draw_text(
                context,
                &label,
                text_x,
                top + SEARCH_ROW_HEIGHT * 0.41,
                fade((1.0, 1.0, 1.0, 0.96), alpha),
            )?;

            if !item.detail.is_empty() {
                set_regular_font(context, 10.5);
                let detail = fit_text(context, &item.detail, max_width)?;
                draw_text(
                    context,
                    &detail,
                    text_x,
                    top + SEARCH_ROW_HEIGHT * 0.75,
                    fade((1.0, 1.0, 1.0, 0.5), alpha),
                )?;
            }

            context.restore()?;
        }
        element += 1;
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
            fade((1.0, 1.0, 1.0, 0.55), presence * state.input_progress()),
        )?;
    }

    Ok(())
}
