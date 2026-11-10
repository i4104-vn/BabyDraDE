//! Logo button with quick system actions

use gtk4::prelude::*;
use gtk4::{Button, GestureClick};
use std::rc::Rc;
use ui_kit::icons::get_icon;
use ui_kit::overlay::context_menu::ContextMenu;

pub struct LogoButton {
    button: Button,
}

impl LogoButton {
    pub fn new() -> Self {
        let button = Button::builder()
            .css_classes(["panel-logo-btn"])
            .tooltip_text("BabyDraDE")
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .build();

        if let Some(cursor) = gtk4::gdk::Cursor::from_name("pointer", None) {
            button.set_cursor(Some(&cursor));
        }

        let icon = get_icon("logo", 16);
        button.set_child(Some(&icon));

        // Create context menu for right-click / system menu
        let menu = Rc::new(
            ContextMenu::new(&button)
                .item("Open Terminal", "utilities-terminal", || {
                    let _ = std::process::Command::new("alacritty")
                        .spawn()
                        .or_else(|_| std::process::Command::new("foot").spawn());
                })
                .item("About BabyDraDE", "info", || {})
                .item("System Settings", "settings", || {})
                .separator()
                .item("Restart", "refresh", || {
                    let _ = std::process::Command::new("systemctl")
                        .arg("reboot")
                        .spawn();
                })
                .item("Shut Down...", "power", || {
                    let _ = std::process::Command::new("systemctl")
                        .arg("poweroff")
                        .spawn();
                }),
        );

        let click = GestureClick::new();
        click.set_button(0); // 1 = Left, 3 = Right

        let menu_handle = menu.clone();
        click.connect_pressed(move |gesture, _, _, _| {
            let btn_num = gesture.current_button();
            if btn_num == 1 || btn_num == 3 {
                menu_handle.popup();
            }
        });
        button.add_controller(click);

        Self { button }
    }

    pub fn widget(&self) -> &Button {
        &self.button
    }
}

impl Default for LogoButton {
    fn default() -> Self {
        Self::new()
    }
}
