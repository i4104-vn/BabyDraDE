use gtk4::Label;

/// A subtle vertical divider line for separating pill groups
pub struct VerticalSeparator {
    label: Label,
}

impl Default for VerticalSeparator {
    fn default() -> Self {
        Self::new()
    }
}

impl VerticalSeparator {
    pub fn new() -> Self {
        let label = Label::builder()
            .label("│")
            .css_classes(["capsule-separator"])
            .build();

        Self { label }
    }

    pub fn widget(&self) -> &Label {
        &self.label
    }
}
