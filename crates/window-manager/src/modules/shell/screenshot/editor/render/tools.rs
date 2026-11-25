//! Drawing tool and shape button wiring for the editor toolbar.

use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::modules::shell::screenshot::editor::models::{EditorState, Tool};
use crate::modules::shell::screenshot::editor::popovers::shape::{
    create_shape_popover, is_shape_tool,
};

/// Wires the drawing tool buttons and the shared shapes button.
///
/// # Arguments
///
/// * `state` - Shared editor state holding the active tool.
/// * `pen` - Pen button.
/// * `shape` - Shared shapes button.
/// * `blur` - Blur button.
/// * `eraser` - Eraser button.
/// * `tool_buttons` - All tool buttons whose selection state is reset.
pub(super) fn wire(
    state: &Rc<RefCell<EditorState>>,
    pen: &gtk4::Button,
    shape: &gtk4::Button,
    blur: &gtk4::Button,
    eraser: &gtk4::Button,
    tool_buttons: &[gtk4::Button],
) {
    let tools = vec![
        (pen.clone(), Tool::Pen),
        (blur.clone(), Tool::Blur),
        (eraser.clone(), Tool::Eraser),
    ];

    for (btn, tool) in tools {
        let state_tool = state.clone();
        let btn_clone = btn.clone();
        let tool_buttons_clone = tool_buttons.to_vec();
        btn.connect_clicked(move |_| {
            state_tool.borrow_mut().current_tool = tool;
            for t_btn in &tool_buttons_clone {
                t_btn.remove_css_class("selected");
            }
            btn_clone.add_css_class("selected");
        });
    }

    let shape_popover = create_shape_popover(shape, state.clone(), tool_buttons);
    let state_shape_btn = state.clone();
    let shape_popover_c = shape_popover.clone();
    let btn_shape_c = shape.clone();
    let tool_buttons_shape = tool_buttons.to_vec();
    shape.connect_clicked(move |_| {
        let active = is_shape_tool(state_shape_btn.borrow().current_tool);
        if active {
            shape_popover_c.popup();
        } else {
            let shape = state_shape_btn.borrow().current_shape;
            state_shape_btn.borrow_mut().current_tool = shape;
            for t_btn in &tool_buttons_shape {
                t_btn.remove_css_class("selected");
            }
            btn_shape_c.add_css_class("selected");
        }
    });
}
