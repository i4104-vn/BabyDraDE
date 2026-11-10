use std::time::Duration;

use cairo::{Context, FontSlant, FontWeight, Operator};

use crate::compositor::{AltTabEntry, AltTabSwitcher};

use super::draw::{
    app_color, draw_app_icon, draw_text, fit_text, lerp, rounded_rect, Rect,
};

pub(crate) const ANIMATION_DURATION: Duration = Duration::from_millis(220);

pub(crate) fn draw_switcher(
    context: &Context,
    width: f64,
    height: f64,
    switcher: &AltTabSwitcher,
) -> Result<(), cairo::Error> {
    if switcher.entries.is_empty() {
        return Ok(());
    }

    let progress: f64 = (switcher.transition_started.elapsed().as_secs_f64()
        / ANIMATION_DURATION.as_secs_f64())
    .clamp(0.0, 1.0);
    let eased = 1.0f64 - (1.0f64 - progress).powi(3);

    context.set_operator(Operator::Source);
    context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    context.paint()?;
    context.set_operator(Operator::Over);

    context.set_source_rgba(0.015, 0.02, 0.04, 0.76);
    context.paint()?;

    let panel_width = 272.0_f64.min((width - 40.0).max(210.0));
    let panel_height = (height - 48.0).clamp(170.0, 700.0);
    let panel_x = 24.0;
    let panel_y = ((height - panel_height) / 2.0).max(12.0);
    rounded_rect(context, panel_x, panel_y, panel_width, panel_height, 22.0);
    context.set_source_rgba(0.055, 0.07, 0.11, 0.97);
    context.fill_preserve()?;
    context.set_source_rgba(1.0, 1.0, 1.0, 0.12);
    context.set_line_width(1.0);
    context.stroke()?;

    context.select_font_face("Sans", FontSlant::Normal, FontWeight::Bold);
    context.set_font_size(15.0);
    draw_text(
        context,
        "Open apps",
        panel_x + 20.0,
        panel_y + 31.0,
        (0.94, 0.96, 1.0, 1.0),
    )?;
    context.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
    context.set_font_size(11.0);
    draw_text(
        context,
        &format!("{} windows", switcher.entries.len()),
        panel_x + 20.0,
        panel_y + 49.0,
        (0.56, 0.62, 0.72, 1.0),
    )?;

    let row_height = 58.0;
    let list_top = panel_y + 64.0;
    let visible_count = (((panel_height - 78.0) / row_height).floor() as usize).max(1);
    let first_visible = switcher
        .selected_index
        .saturating_sub(visible_count / 2)
        .min(switcher.entries.len().saturating_sub(visible_count));
    let last_visible = (first_visible + visible_count).min(switcher.entries.len());
    let visible_row = |index: usize| -> Option<f64> {
        (index >= first_visible && index < last_visible)
            .then_some(list_top + (index - first_visible) as f64 * row_height)
    };

    for (index, entry) in switcher
        .entries
        .iter()
        .enumerate()
        .take(last_visible)
        .skip(first_visible)
    {
        let row_y = visible_row(index).unwrap_or(list_top);
        let selected = index == switcher.selected_index;
        if selected {
            rounded_rect(
                context,
                panel_x + 9.0,
                row_y,
                panel_width - 18.0,
                50.0,
                13.0,
            );
            context.set_source_rgba(0.30, 0.48, 0.82, 0.38);
            context.fill()?;
            rounded_rect(
                context,
                panel_x + 9.0,
                row_y,
                panel_width - 18.0,
                50.0,
                13.0,
            );
            context.set_source_rgba(0.57, 0.70, 1.0, 0.30);
            context.set_line_width(1.0);
            context.stroke()?;
        }

        let icon_color = app_color(&entry.app_name);
        draw_app_icon(
            context,
            entry,
            panel_x + 19.0,
            row_y + 8.0,
            34.0,
            icon_color,
        )?;

        context.select_font_face("Sans", FontSlant::Normal, FontWeight::Bold);
        context.set_font_size(13.0);
        let app_name = fit_text(context, &entry.app_name, panel_width - 84.0)?;
        draw_text(
            context,
            &app_name,
            panel_x + 64.0,
            row_y + 22.0,
            if selected {
                (0.98, 0.99, 1.0, 1.0)
            } else {
                (0.85, 0.88, 0.93, 1.0)
            },
        )?;

        context.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
        context.set_font_size(10.5);
        let title = fit_text(context, &entry.title, panel_width - 84.0)?;
        draw_text(
            context,
            &title,
            panel_x + 64.0,
            row_y + 38.0,
            (0.59, 0.65, 0.74, 1.0),
        )?;
    }

    let Some(entry) = switcher.entries.get(switcher.selected_index) else {
        return Ok(());
    };
    let target_width = (width * 0.48).clamp(300.0, 520.0).min(width - 48.0);
    let target_height = (height * 0.42).clamp(220.0, 330.0).min(height - 48.0);
    let target = Rect {
        x: (width - target_width) / 2.0,
        y: (height - target_height) / 2.0,
        width: target_width,
        height: target_height,
    };
    let source = visible_row(switcher.selected_index)
        .map(|row_y| Rect {
            x: panel_x + 16.0,
            y: row_y + 4.0,
            width: 46.0,
            height: 46.0,
        })
        .unwrap_or(Rect {
            x: panel_x + 16.0,
            y: panel_y + 72.0,
            width: 46.0,
            height: 46.0,
        });
    let card = Rect {
        x: lerp(source.x, target.x, eased),
        y: lerp(source.y, target.y, eased),
        width: lerp(source.width, target.width, eased),
        height: lerp(source.height, target.height, eased),
    };

    draw_selected_card(context, entry, card, eased)?;
    Ok(())
}

fn draw_selected_card(
    context: &Context,
    entry: &AltTabEntry,
    rect: Rect,
    progress: f64,
) -> Result<(), cairo::Error> {
    let corner = (rect.width.min(rect.height) * 0.09).clamp(12.0, 24.0);
    rounded_rect(
        context,
        rect.x + 2.0,
        rect.y + 12.0,
        rect.width,
        rect.height,
        corner,
    );
    context.set_source_rgba(0.0, 0.0, 0.0, 0.38 * progress);
    context.fill()?;

    rounded_rect(context, rect.x, rect.y, rect.width, rect.height, corner);
    context.set_source_rgba(0.075, 0.09, 0.14, 0.97);
    context.fill_preserve()?;
    context.set_source_rgba(1.0, 1.0, 1.0, 0.17);
    context.set_line_width(1.2);
    context.stroke()?;

    let icon_size = (rect.height * 0.34)
        .clamp(52.0, 92.0)
        .min(rect.width * 0.25);
    let icon_x = rect.x + 28.0;
    let icon_y = rect.y + (rect.height - icon_size) / 2.0;
    draw_app_icon(
        context,
        entry,
        icon_x,
        icon_y,
        icon_size,
        app_color(&entry.app_name),
    )?;

    if progress > 0.12 && rect.width > 160.0 {
        let text_x = icon_x + icon_size + 22.0;
        let max_text_width = (rect.x + rect.width - text_x - 24.0).max(40.0);
        context.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
        context.set_font_size(11.0);
        draw_text(
            context,
            "SWITCHING TO",
            text_x,
            rect.y + rect.height * 0.36,
            (0.61, 0.69, 0.83, progress),
        )?;
        context.select_font_face("Sans", FontSlant::Normal, FontWeight::Bold);
        context.set_font_size(24.0_f64.min(rect.height * 0.13));
        let app_name = fit_text(context, &entry.app_name, max_text_width)?;
        draw_text(
            context,
            &app_name,
            text_x,
            rect.y + rect.height * 0.50,
            (0.97, 0.98, 1.0, progress),
        )?;
        context.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
        context.set_font_size(13.0);
        let title = fit_text(context, &entry.title, max_text_width)?;
        draw_text(
            context,
            &title,
            text_x,
            rect.y + rect.height * 0.61,
            (0.66, 0.72, 0.81, progress),
        )?;
    }
    Ok(())
}

