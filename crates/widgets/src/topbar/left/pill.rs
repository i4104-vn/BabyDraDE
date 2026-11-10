use gtk4::prelude::*;
use ui_kit::{PillBox, VerticalSeparator};

use super::app_menu::AppMenu;
use super::logo_button::LogoButton;

pub struct LeftPill {
    pill: PillBox,
    _sep: VerticalSeparator,
    app_menu: AppMenu,
}

impl LeftPill {
    pub fn new() -> Self {
        let pill = PillBox::with_extra_class(4, "left-pill");

        // 1. Logo button
        let logo_btn = LogoButton::new();
        pill.append(logo_btn.widget());

        // 2. Vertical separator
        let sep = VerticalSeparator::new();
        pill.append(sep.widget());

        // 3. Standard AppMenu
        let app_menu = AppMenu::new();
        pill.append(app_menu.widget());

        // Bind separator visibility directly to app_menu visibility:
        // When no app is focused, both AppMenu and separator are completely hidden!
        app_menu
            .widget()
            .bind_property("visible", sep.widget(), "visible")
            .sync_create()
            .build();

        Self {
            pill,
            _sep: sep,
            app_menu,
        }
    }

    pub fn set_active_app(&self, name: &str) {
        self.app_menu.set_active_app(name);
    }

    pub fn widget(&self) -> &gtk4::Box {
        self.pill.widget()
    }
}

impl Default for LeftPill {
    fn default() -> Self {
        Self::new()
    }
}
