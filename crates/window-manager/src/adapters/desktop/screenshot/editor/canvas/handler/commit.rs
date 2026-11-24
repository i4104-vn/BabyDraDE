//! Finalization of drag gestures into committed drawings.

use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::adapters::desktop::screenshot::editor::models::{Drawing, EditorState, Tool};

/// Commits the active drawing when a drag gesture ends.
pub(super) fn finish_drag(
    state: &Rc<RefCell<EditorState>>,
    canvas: &gtk4::DrawingArea,
    toolbar: &gtk4::Box,
    btn_pen: &gtk4::Button,
) {
    let mut s_mut = state.borrow_mut();
    let s = &mut *s_mut;
    match s.current_tool {
        Tool::Select => {
            s.is_selecting = false;
            if s.crop_w > 5.0 && s.crop_h > 5.0 {
                s.current_tool = Tool::Pen;
                btn_pen.add_css_class("selected");
            } else {
                s.has_selection = false;
                s.crop_x = 0.0;
                s.crop_y = 0.0;
                s.crop_w = 0.0;
                s.crop_h = 0.0;
            }
            s.invalidate();
        }
        Tool::Pen => {
            let color = s.current_color;
            let width = s.current_width;
            if let Some(points) = s.active_stroke.take() {
                if points.len() >= 2 {
                    s.drawings.push(Drawing::Stroke {
                        points,
                        color,
                        width,
                    });
                    s.invalidate();
                }
            }
        }
        Tool::Rect => {
            if let Some((x, y, w, h)) = s.active_rect.take() {
                if w > 5.0 && h > 5.0 {
                    s.drawings.push(Drawing::Rect {
                        x,
                        y,
                        w,
                        h,
                        color: s.current_color,
                        width: s.current_width,
                    });
                    s.invalidate();
                }
            }
        }
        Tool::Ellipse => {
            if let Some((x, y, w, h)) = s.active_rect.take() {
                if w > 5.0 && h > 5.0 {
                    s.drawings.push(Drawing::Ellipse {
                        x,
                        y,
                        w,
                        h,
                        color: s.current_color,
                        width: s.current_width,
                    });
                    s.invalidate();
                }
            }
        }
        Tool::Blur => {
            if let Some((x, y, w, h)) = s.active_rect.take() {
                if w > 5.0 && h > 5.0 {
                    s.drawings.push(Drawing::Blur { x, y, w, h });
                    s.invalidate();
                }
            }
        }
        Tool::Line => {
            if let Some((x1, y1, x2, y2)) = s.active_line.take() {
                if (x2 - x1).hypot(y2 - y1) > 8.0 {
                    s.drawings.push(Drawing::Line {
                        x1,
                        y1,
                        x2,
                        y2,
                        color: s.current_color,
                        width: s.current_width,
                    });
                    s.invalidate();
                }
            }
        }
        Tool::Arrow => {
            if let Some((x1, y1, x2, y2)) = s.active_line.take() {
                if (x2 - x1).hypot(y2 - y1) > 8.0 {
                    s.drawings.push(Drawing::Arrow {
                        x1,
                        y1,
                        x2,
                        y2,
                        color: s.current_color,
                        width: s.current_width,
                    });
                    s.invalidate();
                }
            }
        }
        Tool::Eraser => {}
    }

    toolbar.set_visible(s.has_selection);
    drop(s_mut);
    canvas.queue_draw();
}
