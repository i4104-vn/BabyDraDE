//! Drag gesture wiring for canvas editing tools.

use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::commit::finish_drag;
use super::selection::erase_at;
use crate::modules::shell::screenshot::editor::models::{EditorState, Tool};

/// Connects drag begin, update and end callbacks to the canvas.
pub(super) fn setup_drag(
    drawing_area: &gtk4::DrawingArea,
    state: Rc<RefCell<EditorState>>,
    toolbar_wrapper: &gtk4::Box,
    btn_pen: &gtk4::Button,
    anim_generation: Rc<Cell<u64>>,
) {
    let drag_gesture = gtk4::GestureDrag::new();

    let state_begin = state.clone();
    let canvas_begin = drawing_area.clone();
    let toolbar_begin = toolbar_wrapper.clone();
    drag_gesture.connect_drag_begin(move |_, x, y| {
        begin_drag(&state_begin, &canvas_begin, &toolbar_begin, x, y);
    });

    let state_update = state.clone();
    let canvas_update = drawing_area.clone();
    drag_gesture.connect_drag_update(move |_, dx, dy| {
        update_drag(&state_update, &canvas_update, dx, dy);
    });

    let canvas_end = drawing_area.clone();
    let toolbar_end = toolbar_wrapper.clone();
    let btn_pen_end = btn_pen.clone();
    let gen = anim_generation.clone();
    drag_gesture.connect_drag_end(move |_, _, _| {
        finish_drag(&state, &canvas_end, &toolbar_end, &btn_pen_end, &gen);
    });

    drawing_area.add_controller(drag_gesture);
}

/// Starts a selection, stroke, shape or eraser interaction at the drag origin.
fn begin_drag(
    state: &Rc<RefCell<EditorState>>,
    canvas: &gtk4::DrawingArea,
    toolbar: &gtk4::Box,
    start_x: f64,
    start_y: f64,
) {
    let mut s_mut = state.borrow_mut();
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
            toolbar.set_visible(false);
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
    canvas.queue_draw();
}

/// Extends the active selection or in-progress drawing by the drag offset.
fn update_drag(
    state: &Rc<RefCell<EditorState>>,
    canvas: &gtk4::DrawingArea,
    offset_x: f64,
    offset_y: f64,
) {
    let mut s_mut = state.borrow_mut();
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
            }
        }
        Tool::Pen => {
            let start_x = s.drag_start_x;
            let start_y = s.drag_start_y;
            if let Some(points) = &mut s.active_stroke {
                let last = points.last().copied().unwrap_or((0.0, 0.0));
                let next = (start_x + offset_x, start_y + offset_y);
                if (last.0 - next.0).powi(2) + (last.1 - next.1).powi(2) > 9.0 {
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
    canvas.queue_draw();
}
