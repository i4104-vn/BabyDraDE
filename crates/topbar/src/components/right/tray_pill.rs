use gtk4::prelude::*;
use gtk4::{Box, Button, Orientation};

pub struct TrayPill {
    container: Box,
}

impl TrayPill {
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .css_classes(["pill-container", "tray-pill"])
            .tooltip_text("System Tray")
            .build();

        // Tray icon: Notifications
        let notif_btn = Button::builder()
            .label("󰂚")
            .css_classes(["tray-icon"])
            .tooltip_text("Notifications")
            .build();
        container.append(&notif_btn);

        // Tray icon: Clipboard
        let clip_btn = Button::builder()
            .label("󰅍")
            .css_classes(["tray-icon"])
            .tooltip_text("Clipboard")
            .build();
        container.append(&clip_btn);

        // Tray icon: Input Method
        let im_btn = Button::builder()
            .label("EN")
            .css_classes(["tray-icon"])
            .tooltip_text("Input Language: English (US)")
            .build();
        container.append(&im_btn);

        Self { container }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}

