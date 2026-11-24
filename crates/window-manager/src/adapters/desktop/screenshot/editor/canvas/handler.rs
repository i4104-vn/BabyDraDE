//! Canvas gesture and selection handling.

mod click;
mod commit;
mod drag;
mod selection;

use std::cell::RefCell;
use std::rc::Rc;

use crate::adapters::desktop::screenshot::editor::models::EditorState;

pub(crate) use selection::delete_selected;

/// Sets up pointer/mouse gestures on the canvas to handle regional selection,
/// free-hand strokes, shapes, eraser, and right-click drawing editing.
pub(crate) fn setup_editor_gest(
    drawing_area: &gtk4::DrawingArea,
    state: Rc<RefCell<EditorState>>,
    toolbar_wrapper: &gtk4::Box,
    btn_pen: &gtk4::Button,
    style_popover: &gtk4::Popover,
) {
    drag::setup_drag(drawing_area, state.clone(), toolbar_wrapper, btn_pen);
    click::setup_click(drawing_area, state, style_popover);
}
