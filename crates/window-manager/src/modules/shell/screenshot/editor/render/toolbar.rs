//! Floating toolbar and action bar assembly for the screenshot editor.

use animation_gtk::{slide_out_cb_cancelable, SlideDirection};
use gtk4::prelude::*;

use super::models::{ToolbarContext, ToolbarParts};
use crate::modules::shell::screenshot::editor::handle::clipboard::copy_to_clipboard;
use crate::modules::shell::screenshot::editor::handle::drawing::trigger_save;
use crate::modules::shell::screenshot::editor::models::Tool;

/// Creates a flat toolbar button with the given icon.
fn toolbar_button(icon: &str, tooltip: &str) -> gtk4::Button {
    let btn = gtk4::Button::builder()
        .child(&ui_kit::icons::get_icon(icon, 16))
        .build();
    btn.set_tooltip_text(Some(tooltip));
    btn.add_css_class("flat");
    btn.add_css_class("screenshot-toolbar-btn");
    btn
}

/// Creates a circular standalone action button (e.g. Save, Close).
fn circle_action_button(icon: &str, tooltip: &str, extra_class: &str) -> gtk4::Button {
    let btn = gtk4::Button::builder()
        .child(&ui_kit::icons::get_icon(icon, 18))
        .width_request(44)
        .height_request(44)
        .build();
    btn.set_tooltip_text(Some(tooltip));
    btn.add_css_class("flat");
    btn.add_css_class("screenshot-circle-btn");
    btn.add_css_class(extra_class);
    btn
}

/// Appends a styled capsule separator to a toolbar.
///
/// # Arguments
///
/// * `toolbar` - Toolbar box receiving the separator.
fn append_separator(toolbar: &gtk4::Box) {
    let separator = gtk4::Label::new(Some("│"));
    separator.add_css_class("capsule-separator");
    toolbar.append(&separator);
}

/// Wires the reset button that clears annotations and hides the toolbar.
///
/// # Arguments
///
/// * `ctx` - Shared editor widgets and state.
/// * `button` - Reset button to connect.
fn wire_reset(ctx: &ToolbarContext, button: &gtk4::Button) {
    let state_reset = ctx.state.clone();
    let toolbar_wrapper_reset = ctx.wrapper.clone();
    let canvas_reset = ctx.canvas.clone();
    let gen = ctx.anim_generation.clone();
    button.connect_clicked(move |_| {
        let mut s = state_reset.borrow_mut();
        s.has_selection = false;
        s.crop_x = 0.0;
        s.crop_y = 0.0;
        s.crop_w = 0.0;
        s.crop_h = 0.0;
        s.drawings.clear();
        s.selected_drawing = None;
        s.active_stroke = None;
        s.active_rect = None;
        s.active_line = None;
        s.current_tool = Tool::Select;
        s.invalidate();
        drop(s);

        let next_gen = gen.get() + 1;
        gen.set(next_gen);
        let wrapper = toolbar_wrapper_reset.clone();
        slide_out_cb_cancelable(
            toolbar_wrapper_reset.upcast_ref::<gtk4::Widget>(),
            SlideDirection::Down,
            35,
            160,
            true,
            gen.clone(),
            next_gen,
            move || {
                wrapper.set_visible(false);
            },
        );

        canvas_reset.queue_draw();
    });
}

/// Wires the copy, save, and cancel buttons.
///
/// # Arguments
///
/// * `ctx` - Shared editor widgets and state.
/// * `copy` - Copy-to-clipboard button.
/// * `save` - Save-to-file button.
/// * `cancel` - Cancel button.
fn wire_actions(
    ctx: &ToolbarContext,
    copy: &gtk4::Button,
    save: &gtk4::Button,
    cancel: &gtk4::Button,
) {
    let state_copy = ctx.state.clone();
    let win_copy = ctx.window.clone();
    copy.connect_clicked(move |_| {
        if copy_to_clipboard(&state_copy.borrow(), &win_copy) {
            win_copy.close();
        }
    });

    let state_save = ctx.state.clone();
    let win_save = ctx.window.clone();
    save.connect_clicked(move |_| {
        if trigger_save(&state_save.borrow()) {
            win_save.close();
        }
    });

    let win_cancel = ctx.window.clone();
    cancel.connect_clicked(move |_| {
        win_cancel.close();
    });
}

/// Builds and assembles the complete action bar: tools capsule on the left,
/// standalone circular save and close buttons on the right.
///
/// # Arguments
///
/// * `ctx` - Shared editor widgets and state.
///
/// # Returns
///
/// The assembled action bar container plus the widgets needed by canvas gestures.
pub(super) fn build(ctx: &ToolbarContext) -> ToolbarParts {
    // Action bar container with 12px gap between main capsule and right buttons
    let action_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    action_bar.add_css_class("screenshot-action-bar");

    // Left capsule containing selection indicator/reset, drawing tools, and color
    let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    toolbar.add_css_class("screenshot-toolbar");

    let btn_reset = toolbar_button("refresh", &services::trans("screenshot.reset_tooltip"));
    let btn_pen = toolbar_button("edit", &services::trans("screenshot.pen_tooltip"));
    let btn_shape = toolbar_button("rect", &services::trans("screenshot.shape_tooltip"));
    let btn_blur = toolbar_button("blur", &services::trans("screenshot.blur_tooltip"));
    let btn_eraser = toolbar_button("eraser", &services::trans("screenshot.eraser_tooltip"));

    let (color_btn, color_popover) = super::color::build(&ctx.state, &ctx.canvas);
    let tool_buttons = vec![
        btn_pen.clone(),
        btn_shape.clone(),
        btn_blur.clone(),
        btn_eraser.clone(),
    ];

    wire_reset(ctx, &btn_reset);
    super::tools::wire(
        &ctx.state,
        &btn_pen,
        &btn_shape,
        &btn_blur,
        &btn_eraser,
        &tool_buttons,
    );

    let btn_copy = toolbar_button("copy", &services::trans("screenshot.copy_tooltip"));
    let btn_save = circle_action_button("download", &services::trans("screenshot.save_tooltip"), "save");
    let btn_cancel = circle_action_button("close", &services::trans("screenshot.cancel_tooltip"), "close");
    wire_actions(ctx, &btn_copy, &btn_save, &btn_cancel);

    // Assemble tool capsule (left side)
    toolbar.append(&btn_reset);
    append_separator(&toolbar);
    for btn in [&btn_pen, &btn_shape, &btn_blur, &btn_eraser] {
        toolbar.append(btn);
    }
    append_separator(&toolbar);
    toolbar.append(&color_btn);
    append_separator(&toolbar);
    toolbar.append(&btn_copy);

    action_bar.append(&toolbar);

    // Right side: standalone 2 circular buttons (save, close)
    let right_actions = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    right_actions.append(&btn_save);
    right_actions.append(&btn_cancel);

    action_bar.append(&right_actions);

    ToolbarParts {
        action_bar,
        pen: btn_pen,
        color_popover,
    }
}
