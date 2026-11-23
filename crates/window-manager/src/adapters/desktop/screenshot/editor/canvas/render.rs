use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::adapters::desktop::screenshot::editor::handle::drawing::{
    draw_pixelated_rect, render_shape,
};
use crate::adapters::desktop::screenshot::editor::model::{Drawing, EditorState, Tool};

/// Cached static background. Only the screenshot itself is cached: the crop
/// rectangle, dimming and annotations are drawn live so moving the crop never
/// triggers an expensive full-surface rebuild (which showed up as a ~1s stall
/// before the drag preview appeared).
pub(crate) struct LayerCache {
    width: i32,
    height: i32,
    scale: i32,
    surface: cairo::ImageSurface,
}

/// Handle shared between the draw function and gesture controllers.
pub(crate) type SharedCache = Rc<RefCell<Option<LayerCache>>>;

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

/// Dimming layer. With a crop selection the hole is cut out with the EvenOdd
/// fill rule, otherwise the whole screenshot is dimmed. Drawn every frame so
/// it tracks the pointer without touching the cache.
fn draw_dimming(cr: &cairo::Context, s: &EditorState, width: f64, height: f64, has_clip: bool) {
    cr.save().unwrap();
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.45);
    cr.rectangle(0.0, 0.0, width, height);
    if has_clip {
        cr.rectangle(s.crop_x, s.crop_y + s.crop_h, s.crop_w, -s.crop_h);
        cr.set_fill_rule(cairo::FillRule::EvenOdd);
    }
    cr.fill().unwrap();
    cr.restore().unwrap();
}

/// Blue outline of the crop rectangle.
fn draw_crop_border(cr: &cairo::Context, s: &EditorState) {
    if !s.has_selection {
        return;
    }
    cr.save().unwrap();
    cr.set_source_rgba(0.23, 0.51, 0.96, 0.85);
    cr.set_line_width(2.0);
    cr.rectangle(s.crop_x, s.crop_y, s.crop_w, s.crop_h);
    cr.stroke().unwrap();
    cr.restore().unwrap();
}

/// Draws the finished annotations and the right-click highlight. Caller has
/// already clipped to the crop rectangle when one is active.
fn render_annotations(cr: &cairo::Context, s: &EditorState) {
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

/// Renders only the background screenshot into the cache surface. Rebuilt on
/// canvas resize or scale-factor change, never while logic state changes.
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

    let bg_w = s.bg_pixbuf.width() as f64;
    let bg_h = s.bg_pixbuf.height() as f64;
    cr.set_antialias(cairo::Antialias::Best);
    cr.save().unwrap();
    if width > 0 && height > 0 && (bg_w != width as f64 || bg_h != height as f64) {
        cr.scale(width as f64 / bg_w, height as f64 / bg_h);
    }
    cr.set_source_pixbuf(&s.bg_pixbuf, 0.0, 0.0);
    cr.source().set_filter(cairo::Filter::Best);
    cr.paint().unwrap();
    cr.restore().unwrap();

    Some(surface)
}

/// Draws the cached background plus everything that changes while editing.
pub(crate) fn draw_editor_canvas(
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
        Some(c) => c.width != wi || c.height != hi || c.scale != scale_factor,
        None => true,
    };
    if stale {
        if let Some(surface) = render_cache_surface(s, wi, hi, scale_factor) {
            *cache_ref = Some(LayerCache {
                width: wi,
                height: hi,
                scale: scale_factor,
                surface,
            });
        }
    }

    // Blit the cached background 1:1 in device pixels for maximum sharpness.
    if let Some(c) = cache_ref.as_ref() {
        cr.save().unwrap();
        cr.scale(1.0 / c.scale as f64, 1.0 / c.scale as f64);
        cr.set_source_surface(&c.surface, 0.0, 0.0).unwrap();
        cr.source().set_filter(cairo::Filter::Nearest);
        cr.paint().unwrap();
        cr.restore().unwrap();
    }
    drop(cache_ref);

    let has_clip = s.has_selection && s.crop_w > 5.0 && s.crop_h > 5.0;

    // Live dimming (with the crop hole) and outline — no cache rebuild.
    draw_dimming(cr, s, width, height, has_clip);
    draw_crop_border(cr, s);

    // Finished annotations, clipped to the crop selection when present.
    cr.set_antialias(cairo::Antialias::Best);
    if has_clip {
        cr.save().unwrap();
        cr.rectangle(s.crop_x, s.crop_y, s.crop_w, s.crop_h);
        cr.clip();
    }
    render_annotations(cr, s);
    if has_clip {
        cr.restore().unwrap();
    }

    // Live preview of the shape currently being dragged.
    cr.set_antialias(cairo::Antialias::None);
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
            Tool::Rect => render_shape(
                cr,
                &Drawing::Rect {
                    x,
                    y,
                    w,
                    h,
                    color: s.current_color,
                    width: s.current_width,
                },
            ),
            Tool::Ellipse => render_shape(
                cr,
                &Drawing::Ellipse {
                    x,
                    y,
                    w,
                    h,
                    color: s.current_color,
                    width: s.current_width,
                },
            ),
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
