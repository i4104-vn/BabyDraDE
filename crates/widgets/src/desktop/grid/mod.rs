pub mod render;

use gtk4::prelude::*;
use gtk4::{Fixed, GestureClick};
use infrastructure::desktop::fetch_desktop_entries;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use ui_kit::overlay::context_menu::ContextMenu;

type RebuildCallback = Rc<dyn Fn()>;
type RebuildHolder = Rc<std::cell::RefCell<Option<RebuildCallback>>>;

pub fn create_desktop_grid(parent_window: &gtk4::ApplicationWindow) -> (Fixed, Rc<dyn Fn()>) {
    let fixed = Fixed::new();
    fixed.set_hexpand(true);
    fixed.set_vexpand(true);
    fixed.add_css_class("desktop-grid");

    let fixed_rebuild = fixed.clone();
    let parent_win_c = parent_window.clone();

    let rebuild_holder: RebuildHolder = Rc::new(std::cell::RefCell::new(None));
    let holder_c = rebuild_holder.clone();

    let rebuild_fn: Rc<dyn Fn()> = Rc::new(move || {
        // Clear all previous children
        while let Some(child) = fixed_rebuild.first_child() {
            fixed_rebuild.remove(&child);
        }

        let entries = fetch_desktop_entries();
        let win_h = parent_win_c.height().max(600);

        let icon_w = 90.0;
        let icon_h = 100.0;
        let pad_x = 16.0;
        let pad_y = 48.0;

        let max_rows = ((win_h as f64 - pad_y - 20.0) / icon_h).max(1.0) as usize;

        for (idx, entry) in entries.iter().enumerate() {
            let col = idx / max_rows;
            let row = idx % max_rows;

            let x = pad_x + (col as f64) * (icon_w + 12.0);
            let y = pad_y + (row as f64) * icon_h;

            let holder_icon = holder_c.clone();
            let rebuild_self: Rc<dyn Fn()> = Rc::new(move || {
                if let Some(ref f) = *holder_icon.borrow() {
                    f();
                }
            });

            let icon_widget = render::create_icon_widget(entry, rebuild_self);
            fixed_rebuild.put(&icon_widget, x, y);
        }
    });

    *rebuild_holder.borrow_mut() = Some(rebuild_fn.clone());

    rebuild_fn();

    // Rebuild on window resize
    let rebuild_resize = rebuild_fn.clone();
    parent_window.connect_notify_local(Some("default-height"), move |_, _| {
        rebuild_resize();
    });

    // Gestures on background:
    let bg_click = GestureClick::new();
    bg_click.set_button(0);

    let fixed_click = fixed.clone();
    let win_for_menu = parent_window.clone();
    let rebuild_for_menu = rebuild_fn.clone();

    bg_click.connect_pressed(move |g, _, x, y| {
        let btn = g.current_button();
        if btn == 1 {
            // Deselect all
            let mut child = fixed_click.first_child();
            while let Some(w) = child {
                w.remove_css_class("selected");
                child = w.next_sibling();
            }
        } else if btn == 3 {
            show_desktop_context_menu(
                &fixed_click,
                &win_for_menu,
                x,
                y,
                rebuild_for_menu.clone(),
            );
        }
    });
    fixed.add_controller(bg_click);

    (fixed, rebuild_fn)
}

fn show_desktop_context_menu(
    parent: &Fixed,
    parent_win: &gtk4::ApplicationWindow,
    x: f64,
    y: f64,
    rebuild: Rc<dyn Fn()>,
) {
    let rebuild_folder = rebuild.clone();
    let rebuild_doc = rebuild.clone();
    let rebuild_sort = rebuild.clone();
    let win_c = parent_win.clone();

    let menu = ContextMenu::at_coords(parent, x, y)
        .item("New Folder", "folder-new", move || {
            create_desktop_folder();
            rebuild_folder();
        })
        .item("New Document", "edit", move || {
            create_desktop_document();
            rebuild_doc();
        })
        .separator()
        .item("Sort By Name", "view-grid", move || {
            rebuild_sort();
        })
        .item("Change Wallpaper...", "display", move || {
            open_wallpaper_picker(&win_c);
        })
        .separator()
        .item("Open in Terminal", "terminal", || {
            open_terminal_in_desktop();
        });

    menu.popup();
}

fn get_desktop_dir() -> PathBuf {
    let desktop_dir = std::env::var("HOME")
        .map(|h| PathBuf::from(h).join("Desktop"))
        .unwrap_or_else(|_| PathBuf::from("."));
    let _ = std::fs::create_dir_all(&desktop_dir);
    desktop_dir
}

fn create_desktop_folder() {
    let desktop = get_desktop_dir();
    let mut target = desktop.join("New Folder");
    let mut counter = 1;
    while target.exists() {
        counter += 1;
        target = desktop.join(format!("New Folder ({counter})"));
    }
    let _ = std::fs::create_dir(&target);
}

fn create_desktop_document() {
    let desktop = get_desktop_dir();
    let mut target = desktop.join("New Document.txt");
    let mut counter = 1;
    while target.exists() {
        counter += 1;
        target = desktop.join(format!("New Document ({counter}).txt"));
    }
    let _ = std::fs::write(&target, "");
}

fn open_terminal_in_desktop() {
    let desktop = get_desktop_dir();
    let _ = Command::new("alacritty")
        .arg("--working-directory")
        .arg(&desktop)
        .spawn()
        .or_else(|_| {
            Command::new("foot")
                .arg("--working-directory")
                .arg(&desktop)
                .spawn()
        });
}

fn open_wallpaper_picker(parent_win: &gtk4::ApplicationWindow) {
    let dialog = gtk4::FileChooserNative::new(
        Some("Select Wallpaper"),
        Some(parent_win),
        gtk4::FileChooserAction::Open,
        Some("Select"),
        Some("Cancel"),
    );

    let filter = gtk4::FileFilter::new();
    filter.set_name(Some("Image Files (*.png, *.jpg, *.jpeg, *.svg, *.webp)"));
    filter.add_mime_type("image/png");
    filter.add_mime_type("image/jpeg");
    filter.add_mime_type("image/svg+xml");
    filter.add_mime_type("image/webp");
    dialog.add_filter(&filter);

    let win_c = parent_win.clone();
    dialog.connect_response(move |d, response| {
        if response == gtk4::ResponseType::Accept {
            if let Some(file) = d.file() {
                if let Some(path) = file.path() {
                    if let Ok(_dest) = infrastructure::wallpaper::set_wallpaper(&path) {
                        win_c.queue_draw();
                    }
                }
            }
        }
    });

    dialog.show();
}
