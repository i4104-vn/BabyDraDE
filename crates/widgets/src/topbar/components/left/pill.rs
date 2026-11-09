use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, Popover};
use ui_kit::{Icon, PillBox, VerticalSeparator};

pub struct LeftPill {
    pill: PillBox,
}

impl LeftPill {
    pub fn new() -> Self {
        let pill = PillBox::with_extra_class(4, "left-pill");

        // 1. Arch Linux icon button
        let arch_btn = Button::builder()
            .css_classes(["arch-btn"])
            .tooltip_text("BabyDraDE Applications")
            .build();

        let icon = Icon::from_file_or_glyph("assets/icons/arch.svg", "󰣇", 16);
        arch_btn.set_child(Some(&icon));
        pill.append(&arch_btn);

        // 2. Vertical separator
        let sep = VerticalSeparator::new();
        pill.append(sep.widget());

        // 3. Current active application name
        let app_title = Label::builder()
            .label("Code")
            .css_classes(["app-title"])
            .build();
        pill.append(&app_title);

        // 4. Function buttons (first 3 visible as requested: File, Edit, Selection)
        for func in ["File", "Edit", "Selection"] {
            let btn = Button::builder()
                .label(func)
                .css_classes(["func-btn"])
                .build();
            pill.append(&btn);
        }

        // 5. Overflow "More" button (...) with Popover menu containing remaining functions
        let more_btn = Button::builder()
            .label("•••")
            .css_classes(["func-btn", "more-btn"])
            .tooltip_text("More actions")
            .build();

        let popover = Popover::builder()
            .autohide(true)
            .css_classes(["more-popover"])
            .build();

        let popover_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .build();

        for func in ["View", "Go", "Run", "Terminal", "Help"] {
            let item_btn = Button::builder()
                .label(func)
                .css_classes(["func-btn"])
                .build();
            let popover_clone = popover.clone();
            item_btn.connect_clicked(move |_| {
                popover_clone.popdown();
            });
            popover_box.append(&item_btn);
        }

        popover.set_child(Some(&popover_box));
        popover.set_parent(&more_btn);

        let popover_clone = popover.clone();
        more_btn.connect_clicked(move |_| {
            popover_clone.popup();
        });

        pill.append(&more_btn);

        Self { pill }
    }

    pub fn widget(&self) -> &Box {
        self.pill.widget()
    }
}
