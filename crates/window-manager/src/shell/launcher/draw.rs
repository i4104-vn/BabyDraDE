//! All Cairo drawing for the launcher. Shared helpers keep the capsule, icon
//! tiles and label surfaces consistent with the topbar style.

use cairo::Context;
use common::constants::launcher::{HAIRLINE, PANEL_SURFACE};
use gtk4::gdk::prelude::GdkCairoContextExt;
use ui_kit::components::dock::{draw_dock, visual_center, DockLayout, DockSide};
use ui_kit::drawing::{app_color, draw_text, fit_text, paint_rounded, rounded_rect, set_bold_font};

use super::{
    LauncherApp, LABEL_FONT_SIZE, LABEL_GAP, LABEL_HEIGHT, LABEL_MAX_TEXT, LABEL_MAX_WIDTH,
    LABEL_PAD, SLOT,
};

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
    panel: &DockLayout,
    position: f64,
    highlight_position: f64,
    selected: Option<usize>,
    logical_w: f64,
    presence: f64,
    slide_x: f64,
) -> Result<(), cairo::Error> {
    // The capsule, icons, highlight and indicator are shared with the switcher.
    draw_dock(
        context,
        panel,
        apps.len(),
        position,
        highlight_position,
        selected,
        presence,
        slide_x,
        DockSide::Left,
        |ctx, index, cx, cy, size, _| {
            if let Some(app) = apps.get(index) {
                draw_icon(ctx, app, cx, cy, size)
            } else {
                Ok(())
            }
        },
    )?;

    // Left-side content: floating title capsule beside the selected icon.
    if let Some(index) = selected {
        if let Some(app) = apps.get(index) {
            let visual = visual_center(apps.len(), position);
            let active_rel = highlight_position - visual;
            let active_y = panel.center_y + active_rel * SLOT;
            draw_label(
                context,
                &app.name,
                panel.x + slide_x + panel.width + LABEL_GAP,
                active_y,
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
    fn draws_shared_dock_surface() {
        let surface =
            cairo::ImageSurface::create(cairo::Format::ARgb32, 200, 600).expect("surface");
        let context = Context::new(&surface).expect("context");
        let panel = DockLayout::new(7, 300.0, 200.0, DockSide::Left);
        assert!(draw_dock(
            &context,
            &panel,
            7,
            3.0,
            3.0,
            Some(3),
            1.0,
            0.0,
            DockSide::Left,
            |_, _, _, _, _, _| Ok(()),
        )
        .is_ok());
    }

    #[test]
    fn renders_backdrop_gradient() {
        let scale = smithay::utils::Scale::from((1.0, 1.0));
        let pixels = render_backdrop_pixels(400, 800, scale, 400.0, 1.0);
        assert!(pixels.is_some());
        assert_eq!(pixels.unwrap().len(), 400 * 800 * 4);
    }
}
