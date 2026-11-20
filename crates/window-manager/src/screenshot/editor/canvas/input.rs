use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::screenshot::editor::model::{Drawing, EditorState, Tool};

use super::render::SharedCache;

/// Selects the topmost drawing under (`px`, `py`), returning `true` when one was hit.
fn pick_drawing(s: &mut EditorState, px: f64, py: f64) -> bool {
    let found = s.drawings.iter().rposition(|d| d.hit_test(px, py));
    let changed = s.selected_drawing != found;
    s.selected_drawing = found;
    if changed {
        s.invalidate();
    }
    found.is_some()
}

/// Removes the currently selected drawing, keeping indices consistent.
pub fn delete_selected(s: &mut EditorState) {
    if let Some(idx) = s.selected_drawing.take() {
        if idx < s.drawings.len() {
            s.drawings.remove(idx);
            s.invalidate();
        }
    }
}

/// Sets up pointer/mouse gestures on the canvas to handle regional selection,
/// free-hand strokes, shapes, eraser, and right-click drawing editing.
pub fn setup_editor_gest(
    drawing_area: &gtk4::DrawingArea,
    state: Rc<RefCell<EditorState>>,
    toolbar_wrapper: &gtk4::Box,
    btn_pen: &gtk4::Button,
    style_popover: &gtk4::Popover,
) {
    let cache: SharedCache = Rc::new(RefCell::new(None));

    let drag_gesture = gtk4::GestureDrag::new();
    let state_mouse = state.clone();
    let canvas_mouse = drawing_area.clone();
    let toolbar_wrapper_begin = toolbar_wrapper.clone();

    drag_gesture.connect_drag_begin(move |_, start_x, start_y| {
        let mut s_mut = state_mouse.borrow_mut();
        let s = &mut *s_mut;

        if !s.has_selection {
            s.current_tool = Tool::Select;
        }

        if s.has_selection && s.current_tool != Tool::Select {
            let inside_crop = start_x >= s.crop_x
                && start_x <= s.crop_x + s.crop_w
                && start_y >= s.crop_y
                && start_y <= s.crop_y + s.crop_h;
            if !inside_crop {
                return;
            }
        }

        // Any left-drag clears the current selection
        if s.selected_drawing.take().is_some() {
            s.invalidate();
        }

        s.drag_start_x = start_x;
        s.drag_start_y = start_y;

        match s.current_tool {
            Tool::Select => {
                s.is_selecting = true;
                s.has_selection = true;
                s.crop_x = start_x;
                s.crop_y = start_y;
                s.crop_w = 0.0;
                s.crop_h = 0.0;
                toolbar_wrapper_begin.set_visible(false);
                s.invalidate();
            }
            Tool::Pen => {
                s.active_stroke = Some(vec![(start_x, start_y)]);
            }
            Tool::Rect | Tool::Ellipse | Tool::Blur => {
                s.active_rect = Some((start_x, start_y, 0.0, 0.0));
            }
            Tool::Line | Tool::Arrow => {
                s.active_line = Some((start_x, start_y, start_x, start_y));
            }
            Tool::Eraser => {
                erase_at(s, start_x, start_y);
            }
        }
        canvas_mouse.queue_draw();
    });

    let state_mouse_update = state.clone();
    let canvas_mouse_update = drawing_area.clone();
    drag_gesture.connect_drag_update(move |_, offset_x, offset_y| {
        let mut s_mut = state_mouse_update.borrow_mut();
        let s = &mut *s_mut;
        match s.current_tool {
            Tool::Select => {
                if s.is_selecting {
                    let rx = s.drag_start_x.min(s.drag_start_x + offset_x);
                    let ry = s.drag_start_y.min(s.drag_start_y + offset_y);
                    let rw = offset_x.abs();
                    let rh = offset_y.abs();
                    s.crop_x = rx;
                    s.crop_y = ry;
                    s.crop_w = rw;
                    s.crop_h = rh;
                    // The overlay hole must follow the drag, so refresh the cache
                    s.invalidate();
                }
            }
            Tool::Pen => {
                let start_x = s.drag_start_x;
                let start_y = s.drag_start_y;
                if let Some(points) = &mut s.active_stroke {
                    let last = points.last().copied().unwrap_or((0.0, 0.0));
                    let next = (start_x + offset_x, start_y + offset_y);
                    if (last.0 - next.0).powi(2) + (last.1 - next.1).powi(2) > 4.0 {
                        points.push(next);
                    }
                }
            }
            Tool::Rect | Tool::Ellipse | Tool::Blur => {
                let rx = s.drag_start_x.min(s.drag_start_x + offset_x);
                let ry = s.drag_start_y.min(s.drag_start_y + offset_y);
                let rw = offset_x.abs();
                let rh = offset_y.abs();
                s.active_rect = Some((rx, ry, rw, rh));
            }
            Tool::Line | Tool::Arrow => {
                if let Some(line) = &mut s.active_line {
                    line.2 = s.drag_start_x + offset_x;
                    line.3 = s.drag_start_y + offset_y;
                }
            }
            _ => {}
        }
        canvas_mouse_update.queue_draw();
    });

    let state_mouse_end = state.clone();
    let toolbar_wrapper_end = toolbar_wrapper.clone();
    let cache_end = cache.clone();
    let canvas_mouse_end = drawing_area.clone();
    let btn_pen_end = btn_pen.clone();
    drag_gesture.connect_drag_end(move |_, _, _| {
        let mut s_mut = state_mouse_end.borrow_mut();
        let s = &mut *s_mut;
        match s.current_tool {
            Tool::Select => {
                s.is_selecting = false;
                if s.crop_w > 5.0 && s.crop_h > 5.0 {
                    s.current_tool = Tool::Pen;
                    btn_pen_end.add_css_class("selected");
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

        toolbar_wrapper_end.set_visible(s.has_selection);
        drop(s_mut);
        // Content changed: force the next frame to rebuild the cached layer
        cache_end.borrow_mut().take();
        canvas_mouse_end.queue_draw();
    });

    drawing_area.add_controller(drag_gesture);

    // Right-click opens the style popover bound to the drawing under the cursor
    let click_gesture = gtk4::GestureClick::new();
    click_gesture.set_button(3);
    let state_click = state;
    let cache_click = cache;
    let canvas_click = drawing_area.clone();
    let popover_click = style_popover.clone();
    click_gesture.connect_pressed(move |_, _, x, y| {
        let mut s = state_click.borrow_mut();
        if s.has_selection
            && x >= s.crop_x
            && x <= s.crop_x + s.crop_w
            && y >= s.crop_y
            && y <= s.crop_y + s.crop_h
            && pick_drawing(&mut s, x, y)
        {
            drop(s);
            cache_click.borrow_mut().take();
            canvas_click.queue_draw();
            popover_click.popup();
        }
    });
    drawing_area.add_controller(click_gesture);
}

/// Erases the topmost drawing under (`x`, `y`).
fn erase_at(s: &mut EditorState, x: f64, y: f64) {
    if let Some(idx) = s.drawings.iter().rposition(|d| d.hit_test(x, y)) {
        s.drawings.remove(idx);
        s.selected_drawing = match s.selected_drawing {
            Some(sel) if sel > idx => Some(sel - 1),
            Some(sel) if sel == idx => None,
            other => other,
        };
        s.invalidate();
    }
}
