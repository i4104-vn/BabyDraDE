//! Color button and its live indicator for the editor toolbar.

use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::modules::shell::screenshot::editor::models::EditorState;
use crate::modules::shell::screenshot::editor::popovers::color::create_color_popover;

/// Creates the color button, its live indicator, and the color popover.
///
/// # Arguments
///
/// * `state` - Shared editor state read by the indicator.
/// * `canvas` - Drawing area whose color is sampled by the popover.
///
/// # Returns
///
/// The color button and its attached popover.
pub(super) fn build(
    state: &Rc<RefCell<EditorState>>,
    canvas: &gtk4::DrawingArea,
) -> (gtk4::Button, gtk4::Popover) {
    let color_btn = gtk4::Button::new();
    color_btn.set_tooltip_text(Some(&services::trans("screenshot.color_tooltip")));
    color_btn.add_css_class("flat");
    color_btn.add_css_class("screenshot-toolbar-btn");

    let color_dot = gtk4::DrawingArea::new();
    color_dot.set_size_request(16, 16);
    color_btn.set_child(Some(&color_dot));

    let state_indicator = state.clone();
    color_dot.set_draw_func(move |_, cr, w, h| {
        let (r, g, b) = state_indicator.borrow().current_color;
        let cx = w as f64 / 2.0;
        let cy = h as f64 / 2.0;
        let radius = (w.min(h) as f64 / 2.0) - 1.5;
        if radius <= 0.0 {
            return;
        }
        cr.arc(cx, cy, radius, 0.0, 2.0 * std::f64::consts::PI);
        cr.set_source_rgb(r, g, b);
        cr.fill_preserve().unwrap();
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.6);
        cr.set_line_width(1.5);
        cr.stroke().unwrap();
    });

    let popover = create_color_popover(&color_btn, state.clone(), &color_dot, canvas);
    let popover_c = popover.clone();
    color_btn.connect_clicked(move |_| {
        popover_c.popup();
    });

    (color_btn, popover)
}
