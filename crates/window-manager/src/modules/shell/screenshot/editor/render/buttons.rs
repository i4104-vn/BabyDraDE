//! Reusable toolbar button and separator constructors.

use gtk4::prelude::*;

/// Creates a flat toolbar button with the given icon.
pub(super) fn toolbar_button(icon: &str, tooltip: &str) -> gtk4::Button {
    let btn = gtk4::Button::builder()
        .child(&ui_kit::icons::get_icon(icon, 16))
        .build();
    btn.set_tooltip_text(Some(tooltip));
    btn.add_css_class("flat");
    btn.add_css_class("screenshot-toolbar-btn");
    btn
}

/// Creates a circular standalone action button (e.g. Save, Close).
pub(super) fn circle_action_button(icon: &str, tooltip: &str, extra_class: &str) -> gtk4::Button {
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
pub(super) fn append_separator(toolbar: &gtk4::Box) {
    let separator = gtk4::Label::new(Some("│"));
    separator.add_css_class("capsule-separator");
    toolbar.append(&separator);
}
