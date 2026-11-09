//! AppMenu component — Desktop Environment application menu
//!
//! Queries and displays the active application's menu bar over D-Bus (`com.canonical.dbusmenu`).
//! Dynamically builds menu buttons and context menus directly from D-Bus items.

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation};
use infrastructure::system::{activate_dbus_menu_item, fetch_dbus_menu, DbusMenuItem};
use std::cell::RefCell;
use std::rc::Rc;
use ui_kit::overlay::context_menu::ContextMenu;

pub struct AppMenu {
    container: Box,
    menubar_box: Box,
    app_title_label: Label,
    current_app_name: Rc<RefCell<String>>,
    current_service: Rc<RefCell<Option<String>>>,
    current_menu_path: Rc<RefCell<Option<String>>>,
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
        let current_service = Rc::new(RefCell::new(None::<String>));
        let current_menu_path = Rc::new(RefCell::new(None::<String>));

        // 1. Active Application Title Button
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

        // Application Title Context Menu (standard application lifecycle options)
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

        // 2. Dynamic D-Bus Menubar Box (populated strictly via D-Bus GetLayout)
        let menubar_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(2)
            .valign(gtk4::Align::Center)
            .build();
        container.append(&menubar_box);

        Self {
            container,
            menubar_box,
            app_title_label,
            current_app_name,
            current_service,
            current_menu_path,
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

    /// Fetches and rebuilds the menu bar from the application's D-Bus menu service
    pub fn load_dbus_menu(&self, service: &str, path: &str) {
        *self.current_service.borrow_mut() = Some(service.to_string());
        *self.current_menu_path.borrow_mut() = Some(path.to_string());

        let menubar_box_c = self.menubar_box.clone();
        let service_str = service.to_string();
        let path_str = path.to_string();

        // Fetch D-Bus menu on worker thread to prevent blocking GTK main loop
        let (tx, rx) = std::sync::mpsc::channel();
        let s_clone = service_str.clone();
        let p_clone = path_str.clone();
        std::thread::spawn(move || {
            let menu_items = fetch_dbus_menu(&s_clone, &p_clone);
            let _ = tx.send(menu_items);
        });

        glib::timeout_add_local(std::time::Duration::from_millis(20), move || {
            match rx.try_recv() {
                Ok(items_opt) => {
                    // Clear previous menubar items
                    while let Some(child) = menubar_box_c.first_child() {
                        menubar_box_c.remove(&child);
                    }

                    if let Some(items) = items_opt {
                        for top_item in items {
                            if !top_item.visible || top_item.is_separator {
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
                        }
                    }
                    glib::ControlFlow::Break
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
            }
        });
    }

    /// Clears any dynamic D-Bus menubar buttons
    pub fn clear_menu(&self) {
        *self.current_service.borrow_mut() = None;
        *self.current_menu_path.borrow_mut() = None;
        while let Some(child) = self.menubar_box.first_child() {
            self.menubar_box.remove(&child);
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
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

fn attach_dbus_submenu(
    btn: &Button,
    items: &[DbusMenuItem],
    service: &str,
    menu_path: &str,
) {
    let menu = ContextMenu::new(btn);
    let menu = populate_menu_items(menu, items, service, menu_path);

    let menu_rc = Rc::new(menu);
    let click_menu = menu_rc.clone();
    btn.connect_clicked(move |_| {
        click_menu.popup();
    });
}
