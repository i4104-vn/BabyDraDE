use gtk4::prelude::*;
use gtk4::{Box, Orientation, Widget};

/// A rounded capsule/pill container styled with glassmorphism blur and subtle borders
pub struct PillBox {
    container: Box,
}

impl PillBox {
    pub fn new(spacing: i32) -> Self {
        let container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(spacing)
            .css_classes(["pill-container"])
            .build();

        Self { container }
    }

    pub fn with_extra_class(spacing: i32, extra_class: &str) -> Self {
        let pill = Self::new(spacing);
        pill.container.add_css_class(extra_class);
        pill
    }

    pub fn append<W: IsA<Widget>>(&self, widget: &W) {
        self.container.append(widget);
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn set_tooltip(&self, text: &str) {
        self.container.set_tooltip_text(Some(text));
    }
}

