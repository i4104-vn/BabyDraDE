//! Shared notch capsule widget for Dynamic Island features.
//!
//! Ported from `babydra-island/src/island/ui/notch.rs`; the icon helper now
//! comes from BabyDraDE's `ui-kit` instead of `babydra_ui_kit`.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::pango::EllipsizeMode;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, CenterBox, Label, Orientation};
use ui_kit::icons::get_icon;

/// Standardized notch capsule presentation widget.
#[derive(Clone)]
pub struct NotchWidget {
    pub container: GtkBox,
    pub icon_container: GtkBox,
    pub title_label: Label,
    pub value_label: Option<Label>,
    current_icon_name: Rc<RefCell<String>>,
}

impl NotchWidget {
    /// Icon on the left, title in the center.
    pub fn new(icon_name: &str, title: &str) -> Self {
        Self::create(icon_name, title, None)
    }

    /// Icon, title and a status/value label on the right (e.g. `80%`).
    pub fn with_value(icon_name: &str, title: &str, initial_value: &str) -> Self {
        Self::create(icon_name, title, Some(initial_value))
    }

    /// Backwards-compatible alias for [`Self::with_value`].
    pub fn build(icon_name: &str, title: &str, initial_value: &str) -> Self {
        Self::with_value(icon_name, title, initial_value)
    }

    fn create(icon_name: &str, title: &str, initial_value: Option<&str>) -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 0);
        container.add_css_class("notch-content");
        container.set_valign(Align::Center);
        container.set_halign(Align::Fill);
        container.set_hexpand(true);
        container.set_vexpand(true);
        container.set_focusable(false);
        container.set_can_focus(false);

        let center_box = CenterBox::new();
        center_box.set_hexpand(true);
        center_box.set_valign(Align::Center);

        let icon_container = GtkBox::new(Orientation::Horizontal, 0);
        icon_container.set_valign(Align::Center);
        icon_container.set_halign(Align::Start);
        icon_container.set_margin_start(4);
        let icon_widget = get_icon(icon_name, 14);
        icon_widget.set_valign(Align::Center);
        icon_widget.set_halign(Align::Start);
        icon_container.append(&icon_widget);

        let title_label = Label::new(Some(title));
        title_label.add_css_class("notch-player-text");
        title_label.set_valign(Align::Center);
        title_label.set_halign(Align::Center);
        title_label.set_xalign(0.5);
        title_label.set_ellipsize(EllipsizeMode::End);
        title_label.set_single_line_mode(true);

        center_box.set_start_widget(Some(&icon_container));
        center_box.set_center_widget(Some(&title_label));

        let value_label = initial_value.map(|value| {
            let label = Label::new(Some(value));
            label.add_css_class("notch-system-value");
            label.set_valign(Align::Center);
            label.set_halign(Align::End);
            label.set_xalign(1.0);
            label.set_margin_end(4);
            center_box.set_end_widget(Some(&label));
            label
        });

        container.append(&center_box);

        Self {
            container,
            icon_container,
            title_label,
            value_label,
            current_icon_name: Rc::new(RefCell::new(icon_name.to_string())),
        }
    }

    /// Updates the displayed icon only when it changed.
    pub fn set_icon(&self, icon_name: &str) {
        let mut current = self.current_icon_name.borrow_mut();
        if *current != icon_name {
            *current = icon_name.to_string();
            while let Some(child) = self.icon_container.first_child() {
                self.icon_container.remove(&child);
            }
            let icon = get_icon(icon_name, 14);
            icon.set_valign(Align::Center);
            icon.set_halign(Align::Start);
            self.icon_container.append(&icon);
        }
    }

    pub fn set_title(&self, text: &str) {
        self.title_label.set_text(text);
    }

    pub fn set_value(&self, text: &str) {
        if let Some(label) = &self.value_label {
            label.set_text(text);
        }
    }

    pub fn update(&self, icon_name: &str, value_text: &str) {
        self.set_icon(icon_name);
        self.set_value(value_text);
    }

    pub fn update_all(&self, icon_name: &str, title_text: &str, value_text: &str) {
        self.set_icon(icon_name);
        self.set_title(title_text);
        self.set_value(value_text);
    }
}
