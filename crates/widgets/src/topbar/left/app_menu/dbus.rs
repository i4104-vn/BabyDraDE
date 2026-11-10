//! D-Bus AppMenu integration (`com.canonical.dbusmenu`)
//!
//! Handles background service discovery, menu structure fetching,
//! and recursive dynamic context submenu binding.

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Box, Button, PositionType};
use services::system::{
    activate_dbus_menu_item, fetch_dbus_menu, find_app_menu_service, DbusMenuItem,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use ui_kit::overlay::context_menu::ContextMenu;

pub fn clear_container(container: &Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

pub fn clear_menubar(
    menubar_box: &Box,
    current_service: &Rc<RefCell<Option<String>>>,
    current_menu_path: &Rc<RefCell<Option<String>>>,
) {
    *current_service.borrow_mut() = None;
    *current_menu_path.borrow_mut() = None;
    clear_container(menubar_box);
}

pub fn trigger_dbus_menu_discovery(
    app_id: &str,
    process_id: Option<u32>,
    gen: u64,
    generation: Rc<RefCell<u64>>,
    menubar_box: Box,
    service_ref: Rc<RefCell<Option<String>>>,
    path_ref: Rc<RefCell<Option<String>>>,
) {
    let app_id_str = app_id.to_string();
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let found = find_app_menu_service(&app_id_str, process_id);
        let _ = tx.send(found);
    });

    glib::timeout_add_local(Duration::from_millis(25), move || match rx.try_recv() {
        Ok(service_opt) => {
            if *generation.borrow() == gen {
                if let Some((service, path)) = service_opt {
                    *service_ref.borrow_mut() = Some(service.clone());
                    *path_ref.borrow_mut() = Some(path.clone());
                    load_and_render_dbus_menu_if_current(
                        &menubar_box,
                        &service,
                        &path,
                        generation.clone(),
                        gen,
                    );
                } else {
                    clear_menubar(&menubar_box, &service_ref, &path_ref);
                }
            }
            glib::ControlFlow::Break
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
        Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
    });
}

pub fn load_and_render_dbus_menu(menubar_box: &Box, service: &str, path: &str) {
    load_and_render_dbus_menu_internal(menubar_box, service, path, None);
}

fn load_and_render_dbus_menu_if_current(
    menubar_box: &Box,
    service: &str,
    path: &str,
    generation: Rc<RefCell<u64>>,
    expected_generation: u64,
) {
    load_and_render_dbus_menu_internal(
        menubar_box,
        service,
        path,
        Some((generation, expected_generation)),
    );
}

fn load_and_render_dbus_menu_internal(
    menubar_box: &Box,
    service: &str,
    path: &str,
    generation_guard: Option<(Rc<RefCell<u64>>, u64)>,
) {
    let menubar_box_c = menubar_box.clone();
    let service_str = service.to_string();
    let path_str = path.to_string();

    let (tx, rx) = std::sync::mpsc::channel();
    let s_clone = service_str.clone();
    let p_clone = path_str.clone();

    std::thread::spawn(move || {
        let menu_items = fetch_dbus_menu(&s_clone, &p_clone);
        let _ = tx.send(menu_items);
    });

    glib::timeout_add_local(Duration::from_millis(20), move || match rx.try_recv() {
        Ok(items_opt) => {
            if generation_guard
                .as_ref()
                .is_some_and(|(generation, expected)| *generation.borrow() != *expected)
            {
                return glib::ControlFlow::Break;
            }

            clear_container(&menubar_box_c);

            if let Some(items) = items_opt {
                let mut displayed_items = 0;
                let mut overflow_started = false;
                let mut overflow_items = Vec::new();

                for top_item in items {
                    if !top_item.visible {
                        continue;
                    }

                    if top_item.is_separator {
                        if overflow_started
                            && !overflow_items
                                .last()
                                .is_some_and(|item: &DbusMenuItem| item.is_separator)
                        {
                            overflow_items.push(top_item);
                        }
                        continue;
                    }

                    if displayed_items >= 3 {
                        overflow_started = true;
                        overflow_items.push(top_item);
                        continue;
                    }

                    let btn = Button::builder()
                        .label(&top_item.label)
                        .css_classes(["func-btn"])
                        .sensitive(top_item.enabled)
                        .build();

                    if !top_item.children.is_empty() {
                        attach_dbus_submenu(&btn, &top_item.children, &service_str, &path_str);
                    } else {
                        let s = service_str.clone();
                        let p = path_str.clone();
                        let id = top_item.id;
                        btn.connect_clicked(move |_| {
                            activate_dbus_menu_item(&s, &p, id);
                        });
                    }

                    menubar_box_c.append(&btn);
                    displayed_items += 1;
                }

                if overflow_items.last().is_some_and(|item| item.is_separator) {
                    overflow_items.pop();
                }

                if !overflow_items.is_empty() {
                    let more_btn = Button::builder()
                        .label("More ▾")
                        .css_classes(["func-btn"])
                        .build();
                    let menu =
                        build_overflow_menu(&more_btn, &overflow_items, &service_str, &path_str);
                    let menu = Rc::new(menu);
                    let menu_handle = menu.clone();
                    more_btn.connect_clicked(move |_| menu_handle.popup());
                    menubar_box_c.append(&more_btn);
                }
            }
            glib::ControlFlow::Break
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
        Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
    });
}

fn build_overflow_menu(
    anchor: &Button,
    items: &[DbusMenuItem],
    service: &str,
    menu_path: &str,
) -> ContextMenu {
    let menu = ContextMenu::new(anchor).with_position(PositionType::Bottom);
    populate_menu_items(menu, items, service, menu_path)
}

fn populate_menu_items(
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

        if !item.children.is_empty() {
            let sub_items = item.children.clone();
            let svc = service.to_string();
            let path = menu_path.to_string();
            menu = menu.submenu(&item.label, None, move |sub_menu| {
                populate_menu_items(sub_menu, &sub_items, &svc, &path)
            });
        } else {
            let s = service.to_string();
            let p = menu_path.to_string();
            let id = item.id;

            menu = menu.item_sensitive(&item.label, "", item.enabled, move || {
                activate_dbus_menu_item(&s, &p, id);
            });
        }
    }
    menu
}

fn attach_dbus_submenu(btn: &Button, items: &[DbusMenuItem], service: &str, menu_path: &str) {
    let menu = ContextMenu::new(btn).with_position(PositionType::Bottom);
    let menu = populate_menu_items(menu, items, service, menu_path);

    let menu_rc = Rc::new(menu);
    let click_menu = menu_rc.clone();
    btn.connect_clicked(move |_| {
        click_menu.popup();
    });
}
