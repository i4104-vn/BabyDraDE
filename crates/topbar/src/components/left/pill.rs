use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, Popover, Image};

pub struct LeftPill {
    container: Box,
    app_title: Label,
}

impl LeftPill {
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .css_classes(["pill-container", "left-pill"])
            .build();

        // 1. Arch Linux icon button
        let arch_btn = Button::builder()
            .css_classes(["arch-btn"])
            .tooltip_text("BabyDraDE Applications")
            .build();

        // Try loading assets/icons/arch.svg or fallback to stylized icon
        if std::path::Path::new("assets/icons/arch.svg").exists() {
            let img = Image::from_file("assets/icons/arch.svg");
            img.set_pixel_size(16);
            arch_btn.set_child(Some(&img));
        } else {
            let arch_label = Label::new(Some("󰣇"));
            arch_btn.set_child(Some(&arch_label));
        }
        container.append(&arch_btn);

        // 2. Vertical separator
        let sep = Label::builder()
            .label("|")
            .css_classes(["separator"])
            .build();
        container.append(&sep);

        // 3. Current active application name
        let app_title = Label::builder()
            .label("Code")
            .css_classes(["app-title"])
            .build();
        container.append(&app_title);

        // 4. Function buttons (first 3 visible as requested: File, Edit, Selection)
        let primary_functions = ["File", "Edit", "Selection"];
        for func in primary_functions {
            let btn = Button::builder()
                .label(func)
                .css_classes(["func-btn"])
                .build();
            container.append(&btn);
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

        let overflow_functions = ["View", "Go", "Run", "Terminal", "Help"];
        for func in overflow_functions {
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

        container.append(&more_btn);

        Self {
            container,
            app_title,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    #[allow(dead_code)]
    pub fn set_app_title(&self, title: &str) {
        self.app_title.set_label(title);
    }
}
