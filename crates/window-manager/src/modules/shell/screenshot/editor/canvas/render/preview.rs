//! Live preview of the drawing currently being dragged.

use super::annotations::draw_blur_on_canvas;
use crate::modules::shell::screenshot::editor::handle::drawing::render_shape;
use crate::modules::shell::screenshot::editor::models::{Drawing, EditorState, Tool};

/// Draws a live preview of the shape currently being dragged.
pub(super) fn render_preview(cr: &cairo::Context, s: &EditorState) {
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
}
