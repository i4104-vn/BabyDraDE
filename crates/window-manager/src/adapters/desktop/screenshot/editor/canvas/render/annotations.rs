//! Finished annotation and blur rendering on the canvas.

use crate::adapters::desktop::screenshot::editor::handle::drawing::{
    draw_pixelated_rect, render_shape,
};
use crate::adapters::desktop::screenshot::editor::models::{Drawing, EditorState};

/// Renders a blur annotation, mapping canvas-space bounds into pixbuf-space
/// sampling coordinates so the mosaic always samples the correct region.
pub(super) fn draw_blur_on_canvas(
    cr: &cairo::Context,
    s: &EditorState,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
) {
    let bg_w = s.bg_pixbuf.width();
    let bg_h = s.bg_pixbuf.height();
    let sx = if s.canvas_w > 0.0 {
        bg_w as f64 / s.canvas_w
    } else {
        1.0
    };
    let sy = if s.canvas_h > 0.0 {
        bg_h as f64 / s.canvas_h
    } else {
        1.0
    };

    let src_x = (x * sx).floor() as i32;
    let src_y = (y * sy).floor() as i32;
    let src_w = ((x + w) * sx).ceil() as i32 - src_x;
    let src_h = ((y + h) * sy).ceil() as i32 - src_y;
    let src_x = src_x.clamp(0, bg_w - 2);
    let src_y = src_y.clamp(0, bg_h - 2);
    let src_w = src_w.clamp(2, bg_w - src_x);
    let src_h = src_h.clamp(2, bg_h - src_y);

    draw_pixelated_rect(cr, &s.bg_pixbuf, (src_x, src_y, src_w, src_h), (x, y, w, h));
}

/// Draws the finished annotations and the right-click highlight. Caller has
/// already clipped to the crop rectangle when one is active.
pub(super) fn render_annotations(cr: &cairo::Context, s: &EditorState) {
    for drawing in &s.drawings {
        match drawing {
            Drawing::Blur { x, y, w, h } => draw_blur_on_canvas(cr, s, *x, *y, *w, *h),
            other => render_shape(cr, other),
        }
    }

    if let Some(idx) = s.selected_drawing {
        if let Some(d) = s.drawings.get(idx) {
            let (bx, by, bw, bh) = d.bounds();
            cr.set_source_rgba(0.23, 0.51, 0.96, 0.95);
            cr.set_line_width(1.5);
            cr.set_dash(&[6.0, 4.0], 0.0);
            cr.rectangle(bx, by, bw, bh);
            cr.stroke().unwrap();
            cr.set_dash(&[], 0.0);
        }
    }
}
