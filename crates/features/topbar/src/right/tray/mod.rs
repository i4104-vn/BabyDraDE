mod menu;
mod render;

use gtk4::glib;
use gtk4::prelude::*;
use services::tray::{activate_item, get_tray_items, TrayItem};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

pub fn create_tray_widget(window: &gtk4::ApplicationWindow) -> gtk4::Box {
    let tray_container = render::build_tray_container();
    let last_snapshot: Rc<RefCell<Vec<TrayItem>>> = Rc::new(RefCell::new(Vec::new()));

    let tray_container_clone = tray_container.clone();
    let last_snapshot_clone = last_snapshot.clone();
    let window_clone = window.clone();

    glib::timeout_add_local(Duration::from_millis(500), move || {
        let current_items = get_tray_items();

        let needs_rebuild = {
            let last = last_snapshot_clone.borrow();
            *last != current_items
        };

        if needs_rebuild {
            while let Some(child) = tray_container_clone.first_child() {
                tray_container_clone.remove(&child);
            }

            if current_items.is_empty() {
                tray_container_clone.set_visible(false);
            } else {
                tray_container_clone.set_visible(true);
                for item in &current_items {
                    let btn = render::build_tray_button(item);
                    attach_tray_click_gesture(&btn, item, &window_clone);
                    tray_container_clone.append(&btn);
                }
            }

            *last_snapshot_clone.borrow_mut() = current_items;
        }

        glib::ControlFlow::Continue
    });

    tray_container
}

fn attach_tray_click_gesture(
    btn: &gtk4::Button,
    item: &TrayItem,
    window: &gtk4::ApplicationWindow,
) {
    let item = item.clone();
    let btn_c = btn.clone();
    let win_c = window.clone();

    let gesture = gtk4::GestureClick::new();
    gesture.set_button(0);

    gesture.connect_pressed(move |g, _, click_x, click_y| {
        let button_num = g.current_button();
        if button_num == 0 {
            return;
        }

        let (window_x, window_y) = btn_c
            .translate_coordinates(&win_c, click_x, click_y)
            .unwrap_or((click_x, click_y));
        let x = window_x as i32 + crate::PANEL_MARGIN_X;
        let y = window_y as i32 + crate::PANEL_MARGIN_Y;

        if !item.menu_path.is_empty() && button_num != 2 {
            menu::show_menu(&item, &btn_c, x, y, button_num);
        } else {
            activate_item(&item, x, y, button_num);
        }
    });

    btn.add_controller(gesture);
}
