use gtk4::prelude::*;
use gtk4::Label;

/// A small rounded tag / badge with pill styling
pub struct Badge {
    label: Label,
}

impl Badge {
    pub fn new(text: &str) -> Self {
        let label = Label::builder()
            .label(text)
            .css_classes(["badge"])
            .build();

        Self { label }
    }

    pub fn subtle(text: &str) -> Self {
        let badge = Self::new(text);
        badge.label.add_css_class("badge-subtle");
        badge
    }

    pub fn set_text(&self, text: &str) {
        self.label.set_label(text);
    }

    pub fn widget(&self) -> &Label {
        &self.label
    }
}

