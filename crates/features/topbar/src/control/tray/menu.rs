//! Renders the `com.canonical.dbusmenu` menu exposed by a Status Notifier Item.
//!
//! Apps that provide a `Menu` property expect the host to draw the menu itself;
//! only when no menu is exported do we fall back to the item's `ContextMenu`
//! D-Bus method. Styling, hover and dismissal are owned by ui-kit's shared
//! `ContextMenu`, so every context menu in the system behaves identically.

use gtk4::glib;
use gtk4::PositionType;
use services::system::{
    activate_dbus_menu_item, fetch_dbus_menu_with_prepared_submenus, DbusMenuItem,
};
use services::tray::{activate_item, TrayItem};
use std::sync::mpsc;
use std::time::Duration;
use ui_kit::overlay::context_menu::ContextMenu;

pub(super) fn show_menu(item: &TrayItem, button: &gtk4::Button, x: i32, y: i32, button_num: u32) {
    let service = item.service.clone();
    let path = item.menu_path.clone();
    let fallback_item = item.clone();
    let button = button.clone();

    let (tx, rx) = mpsc::channel();
    let fetch_service = service.clone();
    let fetch_path = path.clone();
    std::thread::spawn(move || {
        let _ = tx.send(fetch_dbus_menu_with_prepared_submenus(
            &fetch_service,
            &fetch_path,
        ));
    });

    glib::timeout_add_local(Duration::from_millis(20), move || match rx.try_recv() {
        Ok(Some(items)) if !items.is_empty() => {
            let menu = ContextMenu::new(&button).with_position(PositionType::Bottom);
            present(populate(menu, &items, &service, &path));
            glib::ControlFlow::Break
        }
        Ok(_) => {
            activate_item(&fallback_item, x, y, button_num);
            glib::ControlFlow::Break
        }
        Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
        Err(mpsc::TryRecvError::Disconnected) => {
            activate_item(&fallback_item, x, y, button_num);
            glib::ControlFlow::Break
        }
    });
}

fn present(menu: ContextMenu) {
    menu.popup();
}

fn populate(
    mut menu: ContextMenu,
    items: &[DbusMenuItem],
    service: &str,
    menu_path: &str,
) -> ContextMenu {
    for item in items {
        if !item.visible {
            continue;
        }
        if item.is_separator {
            menu = menu.separator();
            continue;
        }

        if item.has_submenu || !item.children.is_empty() {
            let children = item.children.clone();
            let service = service.to_string();
            let menu_path = menu_path.to_string();
            menu = menu.submenu(&item.label, None, move |submenu| {
                populate(submenu, &children, &service, &menu_path)
            });
        } else {
            let service = service.to_string();
            let menu_path = menu_path.to_string();
            let id = item.id;
            menu = menu.item_sensitive(&item.label, "", item.enabled, move || {
                activate_dbus_menu_item(&service, &menu_path, id);
            });
        }
    }
    menu
}
