use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation};

pub struct DynamicIsland {
    container: Box,
    title_label: Label,
    badge_label: Label,
}

impl DynamicIsland {
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .css_classes(["pill-container", "dynamic-island"])
            .tooltip_text("BabyDraDE Dynamic Island")
            .build();

        // 1. Circular disc/music icon on the left
        let disc_icon = Label::builder()
            .label("♫")
            .css_classes(["island-disc"])
            .build();
        container.append(&disc_icon);

        // 2. Middle label: "Music Player"
        let title_label = Label::builder()
            .label("Music Player")
            .css_classes(["island-title"])
            .build();
        container.append(&title_label);

        // 3. Right percentage badge: "10%"
        let badge_label = Label::builder()
            .label("10%")
            .css_classes(["island-badge"])
            .build();
        container.append(&badge_label);

        Self {
            container,
            title_label,
            badge_label,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    #[allow(dead_code)]
    pub fn set_text(&self, text: &str) {
        self.title_label.set_label(text);
    }

    #[allow(dead_code)]
    pub fn set_percentage(&self, pct: u32) {
        self.badge_label.set_label(&format!("{}%", pct));
    }
}
