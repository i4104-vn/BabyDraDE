use gtk4::prelude::*;
use gtk4::Button;

/// An interactive button with smooth hover/active transitions
pub struct FluentButton {
    button: Button,
}

impl FluentButton {
    pub fn new(label: &str) -> Self {
        let button = Button::builder()
            .label(label)
            .css_classes(["fluent-btn"])
            .build();

        Self { button }
    }

    pub fn with_tooltip(label: &str, tooltip: &str) -> Self {
        let btn = Self::new(label);
        btn.button.set_tooltip_text(Some(tooltip));
        btn
    }

    pub fn widget(&self) -> &Button {
        &self.button
    }
}

