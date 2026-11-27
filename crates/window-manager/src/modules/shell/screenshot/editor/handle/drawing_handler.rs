//! Screenshot save and geometry helpers.

use crate::modules::shell::screenshot::editor::models::{Drawing, EditorState};
use gtk4::gdk::prelude::GdkCairoContextExt;
use services::screenshot::{get_screenshot_path, notify, save_png};

use super::drawing_render::{draw_pixelated_rect, render_shape};

/// Saves the edited crop and notifies the user after a successful write.
pub(crate) fn trigger_save(state: &EditorState) -> bool {
    let Some(surface) = save_cropped_surface(state) else {
        return false;
    };
    let save_path = get_screenshot_path();
    if !save_png(&surface, &save_path) {
        return false;
    }
    let title = services::trans("screenshot.saved_title");
    let message =
        services::trans("screenshot.saved_msg").replace("{}", &format!("{:?}", save_path));
    notify(&title, &message);
    true
}

/// Applies stroke styling shared by every outlined shape.
pub(crate) fn save_cropped_surface(state: &EditorState) -> Option<cairo::ImageSurface> {
    if !state.has_selection || state.crop_w <= 5.0 || state.crop_h <= 5.0 {
        return None;
    }

    let bg_w = state.bg_pixbuf.width();
    let bg_h = state.bg_pixbuf.height();

    let scale_x = if state.canvas_w > 0.0 {
        bg_w as f64 / state.canvas_w
    } else {
        1.0
    };
    let scale_y = if state.canvas_h > 0.0 {
        bg_h as f64 / state.canvas_h
    } else {
        1.0
    };

    let raw_x = ((state.crop_x * scale_x).floor() as i32).clamp(0, bg_w - 1);
    let raw_y = ((state.crop_y * scale_y).floor() as i32).clamp(0, bg_h - 1);
    let raw_w =
        (((state.crop_x + state.crop_w) * scale_x).ceil() as i32).clamp(raw_x + 1, bg_w) - raw_x;
    let raw_h =
        (((state.crop_y + state.crop_h) * scale_y).ceil() as i32).clamp(raw_y + 1, bg_h) - raw_y;

    let sub_pb = state.bg_pixbuf.new_subpixbuf(raw_x, raw_y, raw_w, raw_h);

    let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, raw_w, raw_h).ok()?;
    let cr = cairo::Context::new(&surface).ok()?;

    cr.set_source_pixbuf(&sub_pb, 0.0, 0.0);
    cr.source().set_filter(cairo::Filter::Nearest);
    if let Err(err) = cr.paint() {
        tracing::warn!("cannot paint crop background: {err}");
        return None;
    }

    cr.set_antialias(cairo::Antialias::Best);

    let tx = state.crop_x * scale_x - raw_x as f64;
    let ty = state.crop_y * scale_y - raw_y as f64;
    let line_scale = (scale_x + scale_y) / 2.0;

    for drawing in &state.drawings {
        match drawing {
            Drawing::Blur { x, y, w, h } => {
                let bx = ((x - state.crop_x) * scale_x).floor();
                let by = ((y - state.crop_y) * scale_y).floor();
                let bw = (w * scale_x).ceil();
                let bh = (h * scale_y).ceil();
                // `clamp` panics when min > max; a tiny crop makes these bounds
                // smaller than the minimum mosaic size.
                let sx = (bx as i32).clamp(0, (raw_w - 2).max(0));
                let sy = (by as i32).clamp(0, (raw_h - 2).max(0));
                let sw = (bw as i32).clamp(2, (raw_w - sx).max(2));
                let sh = (bh as i32).clamp(2, (raw_h - sy).max(2));
                draw_pixelated_rect(&cr, &sub_pb, (sx, sy, sw, sh), (bx, by, bw, bh));
            }
            other => {
                if let Err(err) = cr.save() {
                    tracing::warn!("cannot save crop context: {err}");
                    continue;
                }
                cr.translate(-tx, -ty);
                cr.scale(scale_x, scale_y);
                render_shape(&cr, &scale_drawing(other, line_scale));
                if let Err(err) = cr.restore() {
                    tracing::warn!("cannot restore crop context: {err}");
                }
            }
        }
    }

    Some(surface)
}

/// Produces a copy of the drawing with stroke widths multiplied by `line_scale`.
pub(crate) fn scale_drawing(drawing: &Drawing, line_scale: f64) -> Drawing {
    match drawing {
        Drawing::Stroke {
            points,
            color,
            width,
        } => Drawing::Stroke {
            points: points.clone(),
            color: *color,
            width: width * line_scale,
        },
        Drawing::Rect {
            x,
            y,
            w,
            h,
            color,
            width,
        } => Drawing::Rect {
            x: *x,
            y: *y,
            w: *w,
            h: *h,
            color: *color,
            width: width * line_scale,
        },
        Drawing::Ellipse {
            x,
            y,
            w,
            h,
            color,
            width,
        } => Drawing::Ellipse {
            x: *x,
            y: *y,
            w: *w,
            h: *h,
            color: *color,
            width: width * line_scale,
        },
        Drawing::Line {
            x1,
            y1,
            x2,
            y2,
            color,
            width,
        } => Drawing::Line {
            x1: *x1,
            y1: *y1,
            x2: *x2,
            y2: *y2,
            color: *color,
            width: width * line_scale,
        },
        Drawing::Arrow {
            x1,
            y1,
            x2,
            y2,
            color,
            width,
        } => Drawing::Arrow {
            x1: *x1,
            y1: *y1,
            x2: *x2,
            y2: *y2,
            color: *color,
            width: width * line_scale,
        },
        Drawing::Blur { .. } => drawing.clone(),
    }
}
