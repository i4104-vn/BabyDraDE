use gtk4::prelude::*;
use gtk4::{Box, GestureClick, Label, Orientation};
use infrastructure::desktop::DesktopEntry;
use std::process::Command;
use std::rc::Rc;
use ui_kit::icons::get_theme_or_svg_icon;
use ui_kit::overlay::context_menu::ContextMenu;

pub fn create_icon_widget(entry: &DesktopEntry) -> Box {
    let icon_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .css_classes(["desktop-icon"])
        .cursor(&gtk4::gdk::Cursor::from_name("pointer", None).unwrap())
        .build();

    let icon_img = get_theme_or_svg_icon(&entry.icon_name, 48);
    icon_img.add_css_class("desktop-icon-image");
    icon_box.append(&icon_img);

    let label = Label::builder()
        .label(&entry.name)
        .css_classes(["desktop-icon-label"])
        .max_width_chars(12)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .build();
    icon_box.append(&label);

    // Context menu on right click
    let menu = Rc::new(
        ContextMenu::new(&icon_box)
            .item("Open", "folder", {
                let path = entry.path.clone();
                move || {
                    let _ = Command::new("xdg-open").arg(&path).spawn();
                }
            })
            .separator()
            .item("Copy", "copy", || {})
            .item("Cut", "cut", || {})
            .destructive_item("Move to Trash", "trash", || {})
            .separator()
            .item("Properties", "info", || {}),
    );

    let click = GestureClick::new();
    click.set_button(0);

    let path_clone = entry.path.clone();
    let is_dir = entry.is_dir;
    let menu_click = menu.clone();
    let box_click = icon_box.clone();

    click.connect_pressed(move |gesture, n_press, _, _| {
        let btn = gesture.current_button();
        if btn == 1 {
            if n_press == 1 {
                // Toggle selection
                if box_click.has_css_class("selected") {
                    box_click.remove_css_class("selected");
                } else {
                    box_click.add_css_class("selected");
                }
            } else if n_press >= 2 {
                // Double click: open
                if is_dir {
                    let _ = Command::new("alacritty")
                        .arg("--working-directory")
                        .arg(&path_clone)
                        .spawn()
                        .or_else(|_| Command::new("xdg-open").arg(&path_clone).spawn());
                } else {
                    let _ = Command::new("xdg-open").arg(&path_clone).spawn();
                }
            }
        } else if btn == 3 {
            menu_click.popup();
        }
    });

    icon_box.add_controller(click);
    icon_box
}

