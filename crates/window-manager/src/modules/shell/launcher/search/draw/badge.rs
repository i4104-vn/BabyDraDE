//! Badge and icon rendering.

use super::*;

/// Draws an application icon or a small shape-based badge for files and web.
pub(super) fn draw_badge(
    context: &Context,
    item: &SearchItem,
    center_x: f64,
    center_y: f64,
    alpha: f64,
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
            context.paint_with_alpha(alpha)?;
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
                fade((0.36, 0.52, 0.92, 0.95), alpha),
                Some(fade(HAIRLINE, alpha)),
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
                fade((1.0, 1.0, 1.0, 0.95), alpha),
            )
        }
        SearchCategory::Files => {
            if item.is_dir {
                paint_rounded(
                    context,
                    x + 1.0,
                    y + 4.0,
                    size * 0.48,
                    size * 0.26,
                    3.0,
                    fade((0.93, 0.70, 0.28, 0.95), alpha),
                    None,
                )?;
                paint_rounded(
                    context,
                    x,
                    y + size * 0.26,
                    size,
                    size * 0.62,
                    4.0,
                    fade((0.98, 0.78, 0.36, 0.95), alpha),
                    Some(fade(HAIRLINE, alpha)),
                )
            } else {
                paint_rounded(
                    context,
                    x + 3.0,
                    y,
                    size - 7.0,
                    size,
                    4.0,
                    fade((0.62, 0.70, 0.82, 0.95), alpha),
                    Some(fade(HAIRLINE, alpha)),
                )?;
                context.move_to(x + size - 9.0, y + 1.0);
                context.line_to(x + size - 3.0, y + 7.0);
                context.line_to(x + size - 9.0, y + 7.0);
                context.close_path();
                context.set_source_rgba(1.0, 1.0, 1.0, 0.45 * alpha);
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
                fade((0.20, 0.60, 0.52, 0.95), alpha),
                Some(fade(HAIRLINE, alpha)),
            )?;
            let mid = center_y;
            context.set_source_rgba(1.0, 1.0, 1.0, 0.75 * alpha);
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
