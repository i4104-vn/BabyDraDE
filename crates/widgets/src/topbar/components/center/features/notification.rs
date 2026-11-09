//! Notification Toast Banner Feature for Dynamic Island

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation};
use std::cell::Cell;
use std::rc::Rc;
use ui_kit::icons::get_icon;

use super::IslandFeature;

pub struct NotificationFeature {
    container: Box,
    is_active: Rc<Cell<bool>>,
    title_label: Label,
    body_label: Label,
}

impl NotificationFeature {
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .css_classes(["notch-content", "notification-notch"])
            .build();

        let icon = get_icon("bell", 14);
        container.append(&icon);

        let title_label = Label::builder()
            .label("Notification")
            .css_classes(["badge-title"])
            .build();
        container.append(&title_label);

        let body_label = Label::builder()
            .label("")
            .css_classes(["badge-desc"])
            .build();
        container.append(&body_label);

        Self {
            container,
            is_active: Rc::new(Cell::new(false)),
            title_label,
            body_label,
        }
    }

    pub fn show_toast(&self, title: &str, body: &str) {
        self.title_label.set_label(title);
        self.body_label.set_label(body);
        self.is_active.set(true);
    }

    pub fn dismiss(&self) {
        self.is_active.set(false);
    }
}

impl Default for NotificationFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandFeature for NotificationFeature {
    fn id(&self) -> &'static str {
        "notification"
    }

    fn priority(&self) -> u32 {
        100
    }

    fn is_active(&self) -> bool {
        self.is_active.get()
    }

    fn widget(&self) -> gtk4::Widget {
        self.container.clone().upcast()
    }
}
