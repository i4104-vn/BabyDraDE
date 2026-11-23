use cairo::{Context, FontSlant, FontWeight};
use gtk4::gdk::prelude::GdkCairoContextExt;
use ui_kit::drawing::{app_color, draw_text, rounded_rect};

use crate::adapters::compositor::AltTabEntry;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Rect {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
}

impl Rect {
    pub(crate) fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width: width.max(1.0),
            height: height.max(1.0),
        }
    }
}

/// Vẽ badge app bo góc, gắn sát góc dưới-trái của preview, không có padding thừa.
pub(crate) fn draw_app_badge(
    context: &Context,
    entry: &AltTabEntry,
    x: f64,
    y: f64,
    size: f64,
    alpha: f64,
) -> Result<(), cairo::Error> {
    if alpha <= 0.01 {
        return Ok(());
    }

    if let Some(ref pixbuf) = entry.icon_pixbuf {
        let pw = pixbuf.width() as f64;
        let ph = pixbuf.height() as f64;
        if pw > 0.0 && ph > 0.0 {
            context.save()?;
            let corner = size * 0.22;
            rounded_rect(context, x, y, size, size, corner);
            context.set_source_rgba(0.0, 0.0, 0.0, 0.30 * alpha);
            context.fill()?;

            context.translate(x, y);
            context.scale(size / pw, size / ph);
            context.set_source_pixbuf(pixbuf, 0.0, 0.0);
            context.paint_with_alpha(alpha)?;
            context.restore()?;
            return Ok(());
        }
    }

    // Fallback: chữ cái đơn sắc bo góc
    let corner = size * 0.25;
    let color = app_color(&entry.app_name);

    rounded_rect(context, x, y, size, size, corner);
    context.set_source_rgba(0.0, 0.0, 0.0, 0.35 * alpha);
    context.fill()?;

    rounded_rect(context, x, y, size, size, corner);
    context.set_source_rgba(color.0, color.1, color.2, 0.95 * alpha);
    context.fill_preserve()?;

    context.set_source_rgba(1.0, 1.0, 1.0, 0.40 * alpha);
    context.set_line_width(1.0);
    context.stroke()?;

    context.select_font_face("Sans", FontSlant::Normal, FontWeight::Bold);
    context.set_font_size(size * 0.50);
    let initial = entry
        .app_name
        .chars()
        .next()
        .unwrap_or('A')
        .to_uppercase()
        .to_string();
    let extents = context.text_extents(&initial)?;
    let tx = x + (size - extents.x_advance()) / 2.0 - extents.x_bearing();
    let ty = y + (size - extents.height()) / 2.0 - extents.y_bearing();
    draw_text(context, &initial, tx, ty, (1.0, 1.0, 1.0, 0.95 * alpha))
}

#[cfg(test)]
#[path = "../../../tests/adapters/shell/switcher/draw.rs"]
mod tests;
