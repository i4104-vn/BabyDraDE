use gtk4::{Box, Button};
use ui_kit::PillBox;

pub struct TrayPill {
    pill: PillBox,
}

impl TrayPill {
    pub fn new() -> Self {
        let pill = PillBox::with_extra_class(4, "tray-pill");
        pill.set_tooltip("System Tray");

        // Tray icon: Notifications
        let notif_btn = Button::builder()
            .label("󰂚")
            .css_classes(["tray-icon"])
            .tooltip_text("Notifications")
            .build();
        pill.append(&notif_btn);

        // Tray icon: Clipboard
        let clip_btn = Button::builder()
            .label("󰅍")
            .css_classes(["tray-icon"])
            .tooltip_text("Clipboard")
            .build();
        pill.append(&clip_btn);

        // Tray icon: Input Method
        let im_btn = Button::builder()
            .label("EN")
            .css_classes(["tray-icon"])
            .tooltip_text("Input Language: English (US)")
            .build();
        pill.append(&im_btn);

        Self { pill }
    }

    pub fn widget(&self) -> &Box {
        self.pill.widget()
    }
}
