//! Right-click selection of drawing annotations.

use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use super::selection::pick_drawing;
use crate::modules::shell::screenshot::editor::models::EditorState;

/// Wires right-click detection for editing the drawing under the cursor.
pub(super) fn setup_click(
    drawing_area: &gtk4::DrawingArea,
    state: Rc<RefCell<EditorState>>,
    style_popover: &gtk4::Popover,
) {
    // Right-click opens the style popover bound to the drawing under the cursor
    let click_gesture = gtk4::GestureClick::new();
    click_gesture.set_button(3);
    let canvas_click = drawing_area.clone();
    let popover_click = style_popover.clone();
    click_gesture.connect_pressed(move |_, _, x, y| {
        let mut s = state.borrow_mut();
        if s.has_selection
            && x >= s.crop_x
            && x <= s.crop_x + s.crop_w
            && y >= s.crop_y
            && y <= s.crop_y + s.crop_h
            && pick_drawing(&mut s, x, y)
        {
            drop(s);
            canvas_click.queue_draw();
            popover_click.popup();
        }
    });
    drawing_area.add_controller(click_gesture);
}
