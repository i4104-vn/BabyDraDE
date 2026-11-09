pub mod render;

use gtk4::prelude::*;
use gtk4::{Fixed, GestureClick};
use infrastructure::desktop::fetch_desktop_entries;
use std::process::Command;
use std::rc::Rc;
use ui_kit::overlay::context_menu::ContextMenu;

pub fn create_desktop_grid(parent_window: &gtk4::ApplicationWindow) -> (Fixed, Rc<dyn Fn()>) {
    let fixed = Fixed::new();
    fixed.set_hexpand(true);
    fixed.set_vexpand(true);
    fixed.add_css_class("desktop-grid");

    // Empty background context menu on right click
    let empty_menu = Rc::new(
        ContextMenu::new(&fixed)
            .item("New Folder", "folder-new", || {})
            .item("New Document", "edit", || {})
            .separator()
            .item("Sort By Name", "view-grid", || {})
            .item("Change Wallpaper...", "display", || {})
            .separator()
            .item("Open in Terminal", "terminal", || {
                let _ = Command::new("alacritty")
                    .spawn()
                    .or_else(|_| Command::new("foot").spawn());
            }),
    );

    // Gestures on background:
    let bg_click = GestureClick::new();
    bg_click.set_button(0);

    let fixed_click = fixed.clone();
    let empty_menu_click = empty_menu.clone();

    bg_click.connect_pressed(move |g, _, _, _| {
        let btn = g.current_button();
        if btn == 1 {
            // Deselect all
            let mut child = fixed_click.first_child();
            while let Some(w) = child {
                w.remove_css_class("selected");
                child = w.next_sibling();
            }
        } else if btn == 3 {
            empty_menu_click.popup();
        }
    });
    fixed.add_controller(bg_click);

    let fixed_rebuild = fixed.clone();
    let parent_win_c = parent_window.clone();

    let rebuild_fn = Rc::new(move || {
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

            let icon_widget = render::create_icon_widget(entry);
            fixed_rebuild.put(&icon_widget, x, y);
        }
    });

    rebuild_fn();

    // Rebuild on window resize
    let rebuild_resize = rebuild_fn.clone();
    parent_window.connect_notify_local(Some("default-height"), move |_, _| {
        rebuild_resize();
    });

    (fixed, rebuild_fn)
}
