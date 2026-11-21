//! All Cairo drawing for the launcher. Shared helpers keep the capsule, icon
//! tiles and label surfaces consistent with the topbar style.

use cairo::Context;
use gtk4::gdk::prelude::GdkCairoContextExt;
use services::graphics::{
    app_color, docked_panel_path, draw_docked_panel, draw_text, fit_text, paint_rounded,
    rounded_rect, set_bold_font, DockSide,
};

use super::layout::{visual_center, Panel};
use super::{
    LauncherApp, HAIRLINE, INDICATOR_HEIGHT, INDICATOR_WIDTH, LABEL_FONT_SIZE, LABEL_GAP,
    LABEL_HEIGHT, LABEL_MAX_TEXT, LABEL_MAX_WIDTH, LABEL_PAD, PANEL_CORNER_RADIUS,
    PANEL_FILLET_RADIUS, PANEL_SURFACE, SELECTION_SIZE, SLOT,
};

/// Docked panel surface with layered soft shadow, matching the notch style.
fn draw_panel(context: &Context, panel: &Panel) -> Result<(), cairo::Error> {
    draw_docked_panel(
        context,
        panel.x,
        panel.y,
        panel.width,
        panel.height,
        PANEL_CORNER_RADIUS,
        PANEL_FILLET_RADIUS,
        PANEL_SURFACE,
        HAIRLINE,
        DockSide::Left,
    )
}

/// Draws an app icon centred on `(center_x, center_y)`.
fn draw_icon(
    context: &Context,
    app: &LauncherApp,
    center_x: f64,
    center_y: f64,
    size: f64,
) -> Result<(), cairo::Error> {
    let x = center_x - size / 2.0;
    let y = center_y - size / 2.0;

    if let Some(ref pixbuf) = app.icon {
        let (pw, ph) = (pixbuf.width() as f64, pixbuf.height() as f64);
        if pw > 0.0 && ph > 0.0 {
            context.save()?;
            rounded_rect(context, x, y, size, size, size * 0.24);
            context.clip();
            let scale = size / pw.max(ph);
            context.translate(x + (size - pw * scale) / 2.0, y + (size - ph * scale) / 2.0);
            context.scale(scale, scale);
            context.set_source_pixbuf(pixbuf, 0.0, 0.0);
            context.paint()?;
            context.restore()?;
            return Ok(());
        }
    }

    // Fallback: a coloured tile with the app initial.
    let color = app_color(&app.name);
    paint_rounded(
        context,
        x,
        y,
        size,
        size,
        size * 0.24,
        (color.0, color.1, color.2, 0.95),
        Some((1.0, 1.0, 1.0, 0.35)),
    )?;

    let initial = app
        .name
        .chars()
        .next()
        .unwrap_or('A')
        .to_uppercase()
        .to_string();
    set_bold_font(context, size * 0.5);
    let extents = context.text_extents(&initial)?;
    let tx = x + (size - extents.x_advance()) / 2.0 - extents.x_bearing();
    let ty = y + (size - extents.height()) / 2.0 - extents.y_bearing();
    draw_text(context, &initial, tx, ty, (1.0, 1.0, 1.0, 0.95))
}

/// Selected app name in a small floating capsule beside the dock.
fn draw_label(
    context: &Context,
    text: &str,
    x: f64,
    center_y: f64,
    logical_w: f64,
) -> Result<(), cairo::Error> {
    set_bold_font(context, LABEL_FONT_SIZE);

    let text = fit_text(context, text, LABEL_MAX_TEXT)?;
    let width = (context.text_extents(&text)?.x_advance() + LABEL_PAD * 2.0).min(LABEL_MAX_WIDTH);
    let x = x.min((logical_w - 8.0 - width).max(8.0));
    let y = center_y - LABEL_HEIGHT / 2.0;
    let radius = LABEL_HEIGHT / 2.0;

    paint_rounded(
        context,
        x,
        y + 3.0,
        width,
        LABEL_HEIGHT,
        radius,
        (0.0, 0.0, 0.0, 0.22),
        None,
    )?;
    paint_rounded(
        context,
        x,
        y,
        width,
        LABEL_HEIGHT,
        radius,
        PANEL_SURFACE,
        Some(HAIRLINE),
    )?;

    let font = context.font_extents()?;
    let baseline = y + (LABEL_HEIGHT - (font.ascent() + font.descent())) / 2.0 + font.ascent();
    draw_text(
        context,
        &text,
        x + LABEL_PAD,
        baseline,
        (1.0, 1.0, 1.0, 0.95),
    )
}

/// Draws the whole overlay: capsule, selection, icons, indicator and label.
/// `presence` (0..=1) drives the fade, `slide_x` slides the docked capsule in
/// from / out to the left screen edge.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_scene(
    context: &Context,
    apps: &[LauncherApp],
    panel: &Panel,
    position: f64,
    selected: Option<usize>,
    logical_w: f64,
    presence: f64,
    slide_x: f64,
) -> Result<(), cairo::Error> {
    // Shifting the panel rectangle moves the capsule, its clip path, icons,
    // indicator and name label together. The shadow follows automatically.
    let mut scene_panel = *panel;
    scene_panel.x += slide_x;

    // Fade the composited scene as a whole so the panel, icons, label and
    // shadow all appear and disappear together.
    context.push_group();
    let drawn = draw_content(context, apps, &scene_panel, position, selected, logical_w);
    let group = context.pop_group()?;
    context.set_source(&group)?;
    context.paint_with_alpha(presence)?;
    drawn
}

fn draw_content(
    context: &Context,
    apps: &[LauncherApp],
    panel: &Panel,
    position: f64,
    selected: Option<usize>,
    logical_w: f64,
) -> Result<(), cairo::Error> {
    draw_panel(context, panel)?;

    let visual = visual_center(apps.len(), position);
    let selected_center_y = panel.center_y + (position - visual) * SLOT;

    // Icons stay inside the docked panel while their list scrolls.
    context.save()?;
    docked_panel_path(
        context,
        panel.x,
        panel.y,
        panel.width,
        panel.height,
        PANEL_CORNER_RADIUS,
        PANEL_FILLET_RADIUS,
        DockSide::Left,
    )?;
    context.clip();

    if selected.is_some() {
        paint_rounded(
            context,
            panel.x + (panel.width - SELECTION_SIZE) / 2.0,
            selected_center_y - SELECTION_SIZE / 2.0,
            SELECTION_SIZE,
            SELECTION_SIZE,
            17.0,
            (1.0, 1.0, 1.0, 0.09),
            None,
        )?;
    }

    let (first, last) = panel.visible_range(apps.len(), visual);
    for index in first..last {
        let rel = index as f64 - visual;
        let (center_x, center_y, size) = panel.icon_geometry(rel, index as f64 - position);
        if let Some(app) = apps.get(index) {
            draw_icon(context, app, center_x, center_y, size)?;
        }
    }

    // White pill on the inner edge, echoing the topbar workspace dots.
    if selected.is_some() {
        paint_rounded(
            context,
            panel.x + 8.0,
            selected_center_y - INDICATOR_HEIGHT / 2.0,
            INDICATOR_WIDTH,
            INDICATOR_HEIGHT,
            INDICATOR_WIDTH / 2.0,
            (1.0, 1.0, 1.0, 0.92),
            None,
        )?;
    }
    context.restore()?;

    if let Some(index) = selected {
        if let Some(app) = apps.get(index) {
            draw_label(
                context,
                &app.name,
                panel.x + panel.width + LABEL_GAP,
                selected_center_y,
                logical_w,
            )?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::buffer::render_backdrop_pixels;
    use super::*;

    #[test]
    fn fit_text_adds_an_ellipsis_when_too_wide() {
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 8, 8).expect("surface");
        let context = Context::new(&surface).expect("context");
        set_bold_font(&context, LABEL_FONT_SIZE);

        let fitted = fit_text(&context, "A very long application name", 40.0).expect("text");
        assert!(fitted.ends_with('…'));
        assert!(context.text_extents(&fitted).expect("extents").x_advance() <= 40.0);
    }

    #[test]
    fn draws_docked_panel_surface_and_hairline() {
        let surface =
            cairo::ImageSurface::create(cairo::Format::ARgb32, 200, 600).expect("surface");
        let context = Context::new(&surface).expect("context");
        let panel = Panel::new(7, 300.0);
        assert!(draw_panel(&context, &panel).is_ok());
    }

    #[test]
    fn renders_backdrop_gradient() {
        let scale = smithay::utils::Scale::from((1.0, 1.0));
        let pixels = render_backdrop_pixels(400, 800, scale, 400.0, 1.0);
        assert!(pixels.is_some());
        assert_eq!(pixels.unwrap().len(), 400 * 800 * 4);
    }
}
