//! AppMenu component — Standard Desktop Environment application menu

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;
use ui_kit::overlay::context_menu::ContextMenu;

pub struct AppMenu {
    container: Box,
    app_title_label: Label,
    current_app_name: Rc<RefCell<String>>,
}

impl AppMenu {
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .valign(gtk4::Align::Center)
            .build();

        let initial_name = "Desktop".to_string();
        let current_app_name = Rc::new(RefCell::new(initial_name.clone()));

        // Active Application Title Button
        let app_title_btn = Button::builder()
            .css_classes(["app-title-btn", "func-btn"])
            .cursor(&gtk4::gdk::Cursor::from_name("pointer", None).unwrap())
            .build();

        let app_title_label = Label::builder()
            .label(&initial_name)
            .css_classes(["app-title"])
            .build();

        app_title_btn.set_child(Some(&app_title_label));
        container.append(&app_title_btn);

        // Standard Application Context Menu
        let app_menu = Rc::new(
            ContextMenu::new(&app_title_btn)
                .item("About Application", "info", || {})
                .separator()
                .item("Preferences...", "settings", || {})
                .separator()
                .item("Hide", "eye-off", || {})
                .item("Hide Others", "eye", || {})
                .item("Show All", "display", || {})
                .separator()
                .item("Quit", "close", || {}),
        );

        let app_menu_click = app_menu.clone();
        app_title_btn.connect_clicked(move |_| {
            app_menu_click.popup();
        });

        // Standard Menu Bar Items (File, Edit, View, Window, Help)
        let menu_items = [
            ("File", vec!["New Window", "New Tab", "Open...", "Close Window"]),
            ("Edit", vec!["Undo", "Redo", "Cut", "Copy", "Paste"]),
            ("View", vec!["Zoom In", "Zoom Out", "Actual Size", "Toggle Fullscreen"]),
            ("Window", vec!["Minimize", "Zoom", "Bring All to Front"]),
            ("Help", vec!["Documentation", "Report Issue", "About"]),
        ];

        for (label, actions) in menu_items {
            let btn = Button::builder()
                .label(label)
                .css_classes(["func-btn"])
                .build();

            let mut builder = ContextMenu::new(&btn);
            for action in actions {
                builder = builder.item(action, "", || {});
            }
            let sub_menu = Rc::new(builder);

            let sub_menu_click = sub_menu.clone();
            btn.connect_clicked(move |_| {
                sub_menu_click.popup();
            });

            container.append(&btn);
        }

        Self {
            container,
            app_title_label,
            current_app_name,
        }
    }

    /// Dynamically update the active application title
    pub fn set_active_app(&self, app_name: &str) {
        let display_name = if app_name.trim().is_empty() {
            "Desktop"
        } else {
            app_name
        };
        *self.current_app_name.borrow_mut() = display_name.to_string();
        self.app_title_label.set_label(display_name);
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}
