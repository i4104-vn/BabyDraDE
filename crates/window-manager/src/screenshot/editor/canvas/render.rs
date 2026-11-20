use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::screenshot::editor::handle::drawing::{draw_pixelated_rect, render_shape};
use crate::screenshot::editor::model::{Drawing, EditorState, Tool};

/// Cached static scene (background + overlay + finished drawings).
/// Rebuilt only when content version, canvas size, or scale factor changes,
/// so pointer drags only repaint the active shape on top of a cheap blit.
pub struct LayerCache {
    version: u64,
    width: i32,
    height: i32,
    scale: i32,
    surface: cairo::ImageSurface,
}

/// Handle shared between the draw function and gesture controllers.
pub type SharedCache = Rc<RefCell<Option<LayerCache>>>;

/// Renders a blur annotation, mapping canvas-space bounds into pixbuf-space
/// sampling coordinates so the mosaic always samples the correct region.
fn draw_blur_on_canvas(cr: &cairo::Context, s: &EditorState, x: f64, y: f64, w: f64, h: f64) {
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

/// Draws everything that only changes when editor content changes: background
/// screenshot, dark overlay with crop hole, crop border, finished drawings, and
/// the selection highlight.
fn render_scene(cr: &cairo::Context, s: &EditorState, width: f64, height: f64) {
    let bg_w = s.bg_pixbuf.width() as f64;
    let bg_h = s.bg_pixbuf.height() as f64;

    cr.set_antialias(cairo::Antialias::Best);

    // 1. Background screenshot accurately scaled to canvas viewport
    cr.save().unwrap();
    if width > 0.0 && height > 0.0 && (bg_w != width || bg_h != height) {
        cr.scale(width / bg_w, height / bg_h);
    }
    cr.set_source_pixbuf(&s.bg_pixbuf, 0.0, 0.0);
    cr.source().set_filter(cairo::Filter::Best);
    cr.paint().unwrap();
    cr.restore().unwrap();

    // 2. Dark overlay
    let has_clip = s.has_selection && s.crop_w > 5.0 && s.crop_h > 5.0;
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.45);
    if has_clip {
        // Clip out the selection area so it remains bright
        cr.save().unwrap();
        cr.rectangle(0.0, 0.0, width, height);
        cr.rectangle(s.crop_x, s.crop_y + s.crop_h, s.crop_w, -s.crop_h);
        cr.set_fill_rule(cairo::FillRule::EvenOdd);
        cr.fill().unwrap();
        cr.restore().unwrap();

        // Selection border
        cr.set_source_rgba(0.23, 0.51, 0.96, 0.85);
        cr.set_line_width(2.0);
        cr.rectangle(s.crop_x, s.crop_y, s.crop_w, s.crop_h);
        cr.stroke().unwrap();
    } else {
        cr.paint().unwrap();
    }

    // Clip drawings to the crop selection so they never cover the dark overlay
    if has_clip {
        cr.save().unwrap();
        cr.rectangle(s.crop_x, s.crop_y, s.crop_w, s.crop_h);
        cr.clip();
    }

    // 3. Finished annotations
    for drawing in &s.drawings {
        match drawing {
            Drawing::Blur { x, y, w, h } => draw_blur_on_canvas(cr, s, *x, *y, *w, *h),
            other => render_shape(cr, other),
        }
    }

    // 4. Highlight for the drawing selected via right-click
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

    if has_clip {
        cr.restore().unwrap();
    }
}

fn render_cache_surface(
    s: &EditorState,
    width: i32,
    height: i32,
    scale: i32,
) -> Option<cairo::ImageSurface> {
    let surface =
        cairo::ImageSurface::create(cairo::Format::ARgb32, width * scale, height * scale).ok()?;
    let cr = cairo::Context::new(&surface).ok()?;
    cr.scale(scale as f64, scale as f64);
    render_scene(&cr, s, width as f64, height as f64);
    Some(surface)
}

/// Draws the cached static layer plus the in-progress shape being dragged.
pub fn draw_editor_canvas(
    cr: &cairo::Context,
    s: &EditorState,
    width: f64,
    height: f64,
    cache: &SharedCache,
    scale_factor: i32,
) {
    let wi = width.max(1.0) as i32;
    let hi = height.max(1.0) as i32;

    let mut cache_ref = cache.borrow_mut();
    let stale = match cache_ref.as_ref() {
        Some(c) => {
            c.version != s.render_version
                || c.width != wi
                || c.height != hi
                || c.scale != scale_factor
        }
        None => true,
    };
    if stale {
        if let Some(surface) = render_cache_surface(s, wi, hi, scale_factor) {
            *cache_ref = Some(LayerCache {
                version: s.render_version,
                width: wi,
                height: hi,
                scale: scale_factor,
                surface,
            });
        }
    }

    // Blit the cached layer 1:1 in device pixels for maximum sharpness
    if let Some(c) = cache_ref.as_ref() {
        cr.save().unwrap();
        cr.scale(1.0 / c.scale as f64, 1.0 / c.scale as f64);
        cr.set_source_surface(&c.surface, 0.0, 0.0).unwrap();
        cr.source().set_filter(cairo::Filter::Nearest);
        cr.paint().unwrap();
        cr.restore().unwrap();
    }

    // Live preview of the shape currently being drawn
    cr.set_antialias(cairo::Antialias::Best);
    let has_clip = s.has_selection && s.crop_w > 5.0 && s.crop_h > 5.0;
    if has_clip {
        cr.save().unwrap();
        cr.rectangle(s.crop_x, s.crop_y, s.crop_w, s.crop_h);
        cr.clip();
    }

    if let Some(points) = &s.active_stroke {
        if points.len() >= 2 {
            cr.set_source_rgb(s.current_color.0, s.current_color.1, s.current_color.2);
            cr.set_line_width(s.current_width);
            cr.set_line_cap(cairo::LineCap::Round);
            cr.set_line_join(cairo::LineJoin::Round);
            cr.move_to(points[0].0, points[0].1);
            for p in &points[1..] {
                cr.line_to(p.0, p.1);
            }
            cr.stroke().unwrap();
        }
    }

    if let Some((x, y, w, h)) = s.active_rect {
        match s.current_tool {
            Tool::Rect => {
                render_shape(
                    cr,
                    &Drawing::Rect {
                        x,
                        y,
                        w,
                        h,
                        color: s.current_color,
                        width: s.current_width,
                    },
                );
            }
            Tool::Ellipse => {
                render_shape(
                    cr,
                    &Drawing::Ellipse {
                        x,
                        y,
                        w,
                        h,
                        color: s.current_color,
                        width: s.current_width,
                    },
                );
            }
            Tool::Blur => draw_blur_on_canvas(cr, s, x, y, w, h),
            _ => {}
        }
    }

    if let Some((x1, y1, x2, y2)) = s.active_line {
        let drawing = match s.current_tool {
            Tool::Line => Drawing::Line {
                x1,
                y1,
                x2,
                y2,
                color: s.current_color,
                width: s.current_width,
            },
            _ => Drawing::Arrow {
                x1,
                y1,
                x2,
                y2,
                color: s.current_color,
                width: s.current_width,
            },
        };
        render_shape(cr, &drawing);
    }

    if has_clip {
        cr.restore().unwrap();
    }
}
