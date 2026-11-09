use gtk4::{Box, Label};
use ui_kit::{Badge, CircularDisc, PillBox};

pub struct DynamicIsland {
    pill: PillBox,
}

impl DynamicIsland {
    pub fn new() -> Self {
        let pill = PillBox::with_extra_class(6, "dynamic-island");
        pill.set_tooltip("BabyDraDE Dynamic Island");

        // 1. Circular disc/music icon on the left
        let disc_icon = CircularDisc::new("♫");
        pill.append(disc_icon.widget());

        // 2. Middle label: "Music Player"
        let title_label = Label::builder()
            .label("Music Player")
            .css_classes(["island-title"])
            .build();
        pill.append(&title_label);

        // 3. Right percentage badge: "10%"
        let badge = Badge::new("10%");
        pill.append(badge.widget());

        Self { pill }
    }

    pub fn widget(&self) -> &Box {
        self.pill.widget()
    }
}
