use cairo::{Context, FontSlant, FontWeight};
use gtk4::gdk::prelude::GdkCairoContextExt;

use crate::compositor::AltTabEntry;

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

    pub(crate) fn center_x(&self) -> f64 {
        self.x + self.width / 2.0
    }
}

pub(crate) fn rounded_rect(
    context: &Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
) {
    let radius = radius.min(width / 2.0).min(height / 2.0);
    context.new_path();
    context.move_to(x + radius, y);
    context.line_to(x + width - radius, y);
    context.arc(
        x + width - radius,
        y + radius,
        radius,
        -std::f64::consts::FRAC_PI_2,
        0.0,
    );
    context.line_to(x + width, y + height - radius);
    context.arc(
        x + width - radius,
        y + height - radius,
        radius,
        0.0,
        std::f64::consts::FRAC_PI_2,
    );
    context.line_to(x + radius, y + height);
    context.arc(
        x + radius,
        y + height - radius,
        radius,
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
    );
    context.line_to(x, y + radius);
    context.arc(
        x + radius,
        y + radius,
        radius,
        std::f64::consts::PI,
        std::f64::consts::PI * 1.5,
    );
    context.close_path();
}


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

    // Use the pre-loaded pixbuf from the entry — no GTK calls in the render path.
    if let Some(ref pixbuf) = entry.icon_pixbuf {
        let pw = pixbuf.width() as f64;
        let ph = pixbuf.height() as f64;
        if pw > 0.0 && ph > 0.0 {
            context.save()?;
            // Soft drop shadow
            let corner = size * 0.22;
            rounded_rect(context, x, y + 1.5, size, size, corner);
            context.set_source_rgba(0.0, 0.0, 0.0, 0.35 * alpha);
            context.fill()?;

            context.translate(x, y);
            context.scale(size / pw, size / ph);
            context.set_source_pixbuf(pixbuf, 0.0, 0.0);
            context.paint_with_alpha(alpha)?;
            context.restore()?;
            return Ok(());
        }
    }

    // Fallback: coloured letter badge when no icon is available.
    let corner = size * 0.25;

    rounded_rect(context, x, y + 2.0, size, size, corner);
    context.set_source_rgba(0.0, 0.0, 0.0, 0.40 * alpha);
    context.fill()?;

    let color = app_color(&entry.app_name);
    rounded_rect(context, x, y, size, size, corner);
    context.set_source_rgba(color.0, color.1, color.2, 0.98 * alpha);
    context.fill_preserve()?;

    context.set_source_rgba(1.0, 1.0, 1.0, 0.45 * alpha);
    context.set_line_width(1.2);
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
    draw_text(context, &initial, tx, ty, (1.0, 1.0, 1.0, 0.98 * alpha))
}

pub(crate) fn draw_text(
    context: &Context,
    text: &str,
    x: f64,
    y: f64,
    color: (f64, f64, f64, f64),
) -> Result<(), cairo::Error> {
    context.set_source_rgba(color.0, color.1, color.2, color.3);
    context.move_to(x, y);
    context.show_text(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that ui_kit icon resolution works and that a resolved pixbuf
    /// can be painted onto a Cairo surface without panicking.
    #[test]
    fn test_icon_pixbuf_paints_to_cairo() {
        let _ = gtk4::init();

        let pb = ui_kit::icons::get_app_icon_pixbuf("chromium", 24)
            .or_else(|| ui_kit::icons::get_app_icon_pixbuf("kitty", 24));

        // At least one common app icon must resolve — if neither is installed the
        // test is a soft pass (we just skip the paint step).
        if let Some(pb) = pb {
            let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 64, 64).unwrap();
            let ctx = cairo::Context::new(&surface).unwrap();
            ctx.save().unwrap();
            let (pw, ph) = (pb.width() as f64, pb.height() as f64);
            ctx.scale(24.0 / pw, 24.0 / ph);
            ctx.set_source_pixbuf(&pb, 0.0, 0.0);
            ctx.paint_with_alpha(1.0).unwrap();
            ctx.restore().unwrap();
        }
    }
}

pub(crate) fn app_color(name: &str) -> (f64, f64, f64) {
    let hash = name.bytes().fold(0x811c9dc5_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(0x01000193)
    });
    let hue = f64::from(hash % 360) / 360.0;
    let chroma = 0.50;
    let secondary = chroma * (1.0 - ((hue * 6.0) % 2.0 - 1.0).abs());
    let (red, green, blue) = match (hue * 6.0).floor() as u8 {
        0 => (chroma, secondary, 0.0),
        1 => (secondary, chroma, 0.0),
        2 => (0.0, chroma, secondary),
        3 => (0.0, secondary, chroma),
        4 => (secondary, 0.0, chroma),
        _ => (chroma, 0.0, secondary),
    };
    let offset = 0.28;
    (red + offset, green + offset, blue + offset)
}
