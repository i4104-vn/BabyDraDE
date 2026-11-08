use gtk4::Label;

/// A circular disc container (e.g. vinyl music player, notification badge, avatar container)
pub struct CircularDisc {
    label: Label,
}

impl CircularDisc {
    pub fn new(glyph: &str) -> Self {
        let label = Label::builder()
            .label(glyph)
            .css_classes(["circular-disc"])
            .build();

        Self { label }
    }

    pub fn set_glyph(&self, glyph: &str) {
        self.label.set_label(glyph);
    }

    pub fn widget(&self) -> &Label {
        &self.label
    }
}
