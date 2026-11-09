//! AppMenu component — Desktop Environment application menu
//!
//! Queries and displays the active application's menu bar over D-Bus (`com.canonical.dbusmenu`).
//! Dynamically builds menu buttons and context menus directly from D-Bus items and controls
//! active window states (minimize, fullscreen, close). Completely hidden when no window is active.

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation};
use infrastructure::system::{
    activate_dbus_menu_item, close_window_by_app, fetch_dbus_menu, find_app_menu_service,
    get_active_window, get_app_display_name, minimize_active_window,
    toggle_fullscreen_active_window, ActiveWindowInfo, DbusMenuItem,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use ui_kit::overlay::context_menu::ContextMenu;

pub struct AppMenu {
    container: Box,
    menubar_box: Box,
    app_title_label: Label,
    current_app_info: Rc<RefCell<ActiveWindowInfo>>,
    current_service: Rc<RefCell<Option<String>>>,
    current_menu_path: Rc<RefCell<Option<String>>>,
    generation: Rc<RefCell<u64>>,
}

impl AppMenu {
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .valign(gtk4::Align::Center)
            .build();

        // 1. Initial active application query
        let initial_info = get_active_window().unwrap_or_default();
        let has_active_window = !initial_info.is_empty();
        container.set_visible(has_active_window);

        let initial_name = if has_active_window {
            get_app_display_name(&initial_info.app_id, &initial_info.title)
        } else {
            String::new()
        };

        let current_app_info = Rc::new(RefCell::new(initial_info));
        let current_service = Rc::new(RefCell::new(None::<String>));
        let current_menu_path = Rc::new(RefCell::new(None::<String>));
        let generation = Rc::new(RefCell::new(0u64));

        // 2. Active Application Title Button
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

        // 3. Dynamic Application Title Context Menu (Real Window Operations)
        let app_info_click = current_app_info.clone();
        let title_btn_ref = app_title_btn.clone();
        app_title_btn.connect_clicked(move |_| {
            let info = app_info_click.borrow().clone();
            if info.is_empty() {
                return;
            }

            let display_title = get_app_display_name(&info.app_id, &info.title);
            let app_id_close = info.app_id.clone();
            let title_close = info.title.clone();

            let menu = ContextMenu::new(&title_btn_ref)
                .item_sensitive(&display_title, "info", false, || {})
                .separator()
                .item("Minimize Window", "eye-off", || {
                    minimize_active_window();
                })
                .item("Toggle Fullscreen", "display", || {
                    toggle_fullscreen_active_window();
                })
                .separator()
                .item("Close Window", "close", move || {
                    close_window_by_app(&app_id_close, &title_close);
                });
            menu.popup();
        });

        // 4. Dynamic D-Bus Menubar Box (populated strictly via D-Bus GetLayout)
        let menubar_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(2)
            .valign(gtk4::Align::Center)
            .build();
        container.append(&menubar_box);

        // 5. Setup periodic polling for real-time active window detection
        let container_poll = container.clone();
        let app_info_poll = current_app_info.clone();
        let label_poll = app_title_label.clone();
        let menubar_poll = menubar_box.clone();
        let service_poll = current_service.clone();
        let path_poll = current_menu_path.clone();
        let gen_poll = generation.clone();

        // Initial D-Bus menu query if active window exists
        {
            let info = app_info_poll.borrow().clone();
            if !info.is_empty() {
                trigger_dbus_menu_discovery(
                    &info.app_id,
                    0,
                    gen_poll.clone(),
                    menubar_poll.clone(),
                    service_poll.clone(),
                    path_poll.clone(),
                );
            }
        }

        glib::timeout_add_local(Duration::from_millis(150), move || {
            let latest = get_active_window().unwrap_or_default();
            let changed = {
                let cur = app_info_poll.borrow();
                cur.app_id != latest.app_id || cur.title != latest.title
            };

            if changed {
                *app_info_poll.borrow_mut() = latest.clone();
                let has_app = !latest.is_empty();
                container_poll.set_visible(has_app);

                let next_gen = *gen_poll.borrow() + 1;
                *gen_poll.borrow_mut() = next_gen;

                if has_app {
                    let display_name = get_app_display_name(&latest.app_id, &latest.title);
                    label_poll.set_label(&display_name);
                    trigger_dbus_menu_discovery(
                        &latest.app_id,
                        next_gen,
                        gen_poll.clone(),
                        menubar_poll.clone(),
                        service_poll.clone(),
                        path_poll.clone(),
                    );
                } else {
                    label_poll.set_label("");
                    clear_menubar(&menubar_poll, &service_poll, &path_poll);
                }
            }

            glib::ControlFlow::Continue
        });

        Self {
            container,
            menubar_box,
            app_title_label,
            current_app_info,
            current_service,
            current_menu_path,
            generation,
        }
    }

    /// Dynamically update the active application title
    pub fn set_active_app(&self, app_name: &str) {
        let is_empty = app_name.trim().is_empty() || app_name.eq_ignore_ascii_case("desktop");
        self.container.set_visible(!is_empty);
        if !is_empty {
            self.app_title_label.set_label(app_name);
            *self.current_app_info.borrow_mut() = ActiveWindowInfo {
                app_id: app_name.to_string(),
                title: app_name.to_string(),
            };
        } else {
            self.app_title_label.set_label("");
            *self.current_app_info.borrow_mut() = ActiveWindowInfo::default();
            self.clear_menu();
        }
        let next_gen = *self.generation.borrow() + 1;
        *self.generation.borrow_mut() = next_gen;
    }

    /// Returns the currently active window information
    pub fn active_window(&self) -> ActiveWindowInfo {
        self.current_app_info.borrow().clone()
    }

    /// Fetches and rebuilds the menu bar from the application's D-Bus menu service
    pub fn load_dbus_menu(&self, service: &str, path: &str) {
        *self.current_service.borrow_mut() = Some(service.to_string());
        *self.current_menu_path.borrow_mut() = Some(path.to_string());
        load_and_render_dbus_menu(&self.menubar_box, service, path);
    }

    /// Clears any dynamic D-Bus menubar buttons
    pub fn clear_menu(&self) {
        clear_menubar(&self.menubar_box, &self.current_service, &self.current_menu_path);
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}

fn clear_menubar(
    menubar_box: &Box,
    current_service: &Rc<RefCell<Option<String>>>,
    current_menu_path: &Rc<RefCell<Option<String>>>,
) {
    *current_service.borrow_mut() = None;
    *current_menu_path.borrow_mut() = None;
    while let Some(child) = menubar_box.first_child() {
        menubar_box.remove(&child);
    }
}

fn trigger_dbus_menu_discovery(
    app_id: &str,
    gen: u64,
    generation: Rc<RefCell<u64>>,
    menubar_box: Box,
    service_ref: Rc<RefCell<Option<String>>>,
    path_ref: Rc<RefCell<Option<String>>>,
) {
    let app_id_str = app_id.to_string();
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let found = find_app_menu_service(&app_id_str);
        let _ = tx.send(found);
    });

    glib::timeout_add_local(Duration::from_millis(25), move || {
        match rx.try_recv() {
            Ok(service_opt) => {
                if *generation.borrow() == gen {
                    if let Some((service, path)) = service_opt {
                        *service_ref.borrow_mut() = Some(service.clone());
                        *path_ref.borrow_mut() = Some(path.clone());
                        load_and_render_dbus_menu(&menubar_box, &service, &path);
                    } else {
                        clear_menubar(&menubar_box, &service_ref, &path_ref);
                    }
                }
                glib::ControlFlow::Break
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
        }
    });
}

fn load_and_render_dbus_menu(menubar_box: &Box, service: &str, path: &str) {
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

    glib::timeout_add_local(Duration::from_millis(20), move || {
        match rx.try_recv() {
            Ok(items_opt) => {
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
