//! Left topbar cluster: a circular logo button and a separate app menu pill.

use gtk4::prelude::*;
use gtk4::{Box, Orientation};
use ui_kit::PillBox;

use super::super::app_menu::AppMenu;
use super::logo_button::LogoButton;

pub struct LeftPill {
    container: Box,
    app_menu: AppMenu,
}

impl LeftPill {
    pub fn new() -> Self {
        let container = Box::builder()
            .css_classes(["left-cluster"])
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .valign(gtk4::Align::Center)
            .build();

        // 1. Circular logo button — always visible, independent from the app pill.
        let logo_button = LogoButton::new();
        container.append(logo_button.widget());

        // 2. Separate rounded pill holding the active app title and its menu bar.
        let app_pill = PillBox::with_extra_class(0, "left-app-pill");
        let app_menu = AppMenu::new();
        app_pill.append(app_menu.widget());
        container.append(app_pill.widget());

        // Hide the whole app pill when there is no active window, so an empty
        // capsule never lingers next to the logo.
        app_menu
            .widget()
            .bind_property("visible", app_pill.widget(), "visible")
            .sync_create()
            .build();

        Self {
            container,
            app_menu,
        }
    }

    pub fn set_active_app(&self, name: &str) {
        self.app_menu.set_active_app(name);
    }

    pub fn widget(&self) -> &gtk4::Box {
        &self.container
    }
}

impl Default for LeftPill {
    fn default() -> Self {
        Self::new()
    }
}
