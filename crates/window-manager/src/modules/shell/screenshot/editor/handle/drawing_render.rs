//! Cairo shape rendering for the screenshot editor.

use crate::modules::shell::screenshot::editor::models::Drawing;
use gtk4::gdk::prelude::GdkCairoContextExt;

pub(crate) fn apply_stroke_style(cr: &cairo::Context, color: (f64, f64, f64), width: f64) {
    cr.set_source_rgb(color.0, color.1, color.2);
    cr.set_line_width(width);
    cr.set_line_cap(cairo::LineCap::Round);
    cr.set_line_join(cairo::LineJoin::Round);
}

/// Renders a single non-blur drawing with its geometry already mapped into the
/// current context space. Used by both the live canvas and the final export so
/// they can never diverge.
pub(crate) fn render_shape(cr: &cairo::Context, drawing: &Drawing) {
    match drawing {
        Drawing::Stroke {
            points,
            color,
            width,
        } => {
            if points.len() < 2 {
                return;
            }
            apply_stroke_style(cr, *color, *width);
            cr.move_to(points[0].0, points[0].1);
            for p in &points[1..] {
                cr.line_to(p.0, p.1);
            }
            cr.stroke().unwrap();
        }
        Drawing::Rect {
            x,
            y,
            w,
            h,
            color,
            width,
        } => {
            apply_stroke_style(cr, *color, *width);
            cr.rectangle(*x, *y, *w, *h);
            cr.stroke().unwrap();
        }
        Drawing::Ellipse {
            x,
            y,
            w,
            h,
            color,
            width,
        } => {
            if *w <= 0.0 || *h <= 0.0 {
                return;
            }
            apply_stroke_style(cr, *color, *width);
            const SEGMENTS: usize = 72;
            let cx = x + w / 2.0;
            let cy = y + h / 2.0;
            let rx = w / 2.0;
            let ry = h / 2.0;
            for i in 0..=SEGMENTS {
                let a = (i as f64) * 2.0 * std::f64::consts::PI / SEGMENTS as f64;
                let px = cx + rx * a.cos();
                let py = cy + ry * a.sin();
                if i == 0 {
                    cr.move_to(px, py);
                } else {
                    cr.line_to(px, py);
                }
            }
            cr.stroke().unwrap();
        }
        Drawing::Line {
            x1,
            y1,
            x2,
            y2,
            color,
            width,
        } => {
            apply_stroke_style(cr, *color, *width);
            cr.move_to(*x1, *y1);
            cr.line_to(*x2, *y2);
            cr.stroke().unwrap();
        }
        Drawing::Arrow {
            x1,
            y1,
            x2,
            y2,
            color,
            width,
        } => {
            apply_stroke_style(cr, *color, *width);
            let angle = (y2 - y1).atan2(x2 - x1);
            let head = (width * 4.5).clamp(12.0, 40.0);
            let spread = std::f64::consts::PI / 7.0;
            cr.move_to(*x1, *y1);
            cr.line_to(x2 - head * angle.cos(), y2 - head * angle.sin());
            cr.stroke().unwrap();
            for dir in [-1.0, 1.0] {
                let a = angle + std::f64::consts::PI + dir * spread;
                cr.move_to(*x2, *y2);
                cr.line_to(x2 + head * a.cos(), y2 + head * a.sin());
                cr.stroke().unwrap();
            }
        }
        Drawing::Blur { .. } => unreachable!("blur is rendered by the caller"),
    }
}

/// Draws a pixelated mosaic filter inside the target rectangle bounds.
pub(crate) fn draw_pixelated_rect(
    cr: &cairo::Context,
    bg_pixbuf: &gdk_pixbuf::Pixbuf,
    src: (i32, i32, i32, i32),
    dest: (f64, f64, f64, f64),
) {
    let (src_x, src_y, src_w, src_h) = src;
    let (dest_x, dest_y, dest_w, dest_h) = dest;
    if src_w < 2 || src_h < 2 || dest_w <= 5.0 || dest_h <= 5.0 {
        return;
    }

    cr.save().unwrap();
    cr.rectangle(dest_x, dest_y, dest_w, dest_h);
    cr.clip();

    const MOSAIC_TARGET: f64 = 10.0;
    let sw = ((dest_w / MOSAIC_TARGET).round() as i32).clamp(2, src_w);
    let sh = ((dest_h / MOSAIC_TARGET).round() as i32).clamp(2, src_h);

    let sub_pb = bg_pixbuf.new_subpixbuf(src_x, src_y, src_w, src_h);
    if let Some(scaled_pb) = sub_pb.scale_simple(sw, sh, gdk_pixbuf::InterpType::Hyper) {
        let sx = dest_w / sw as f64;
        let sy = dest_h / sh as f64;
        cr.scale(sx, sy);
        cr.set_source_pixbuf(&scaled_pb, dest_x / sx, dest_y / sy);
        cr.source().set_filter(cairo::Filter::Nearest);
        cr.paint().unwrap();
    }

    cr.restore().unwrap();
}
