//! Default Idle Feature for Dynamic Island

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation};

use super::IslandFeature;

pub struct DefaultFeature {
    container: Box,
}

impl DefaultFeature {
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .css_classes(["notch-content", "default-notch"])
            .build();

        let icon = ui_kit::icons::get_icon("logo", 14);
        container.append(&icon);

        let label = Label::builder()
            .label("BabyDraDE")
            .css_classes(["island-title", "notch-player-text"])
            .build();
        container.append(&label);

        Self { container }
    }
}

impl Default for DefaultFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandFeature for DefaultFeature {
    fn id(&self) -> &'static str {
        "default"
    }

    fn priority(&self) -> u32 {
        0
    }

    fn is_active(&self) -> bool {
        true
    }

    fn widget(&self) -> gtk4::Widget {
        self.container.clone().upcast()
    }
}

