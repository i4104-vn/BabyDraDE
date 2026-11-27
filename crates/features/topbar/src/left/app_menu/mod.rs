//! AppMenu component — Desktop Environment application menu
//!
//! Queries and displays the active application's menu bar over D-Bus (`com.canonical.dbusmenu`).
//! Dynamically builds menu buttons and context menus directly from D-Bus items and controls
//! active window states (minimize, fullscreen, close). Completely hidden when no window is active.

pub mod dbus;
pub mod menu;

use dbus::{clear_menubar, load_and_render_dbus_menu, trigger_dbus_menu_discovery};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Box, Button, GestureClick, Label, Orientation};
use menu::show_window_action_menu;
use services::system::{get_active_window, get_app_display_name, ActiveWindowInfo};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use ui_kit::VerticalSeparator;

pub struct AppMenu {
    container: Box,
    sep: VerticalSeparator,
    menubar_box: Box,
    app_title_icon: gtk4::Image,
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
            .build();

        if let Some(cursor) = gtk4::gdk::Cursor::from_name("pointer", None) {
            app_title_btn.set_cursor(Some(&cursor));
        }

        let app_title_icon = gtk4::Image::new();
        app_title_icon.set_pixel_size(16);
        app_title_icon.add_css_class("app-title-icon");
        update_app_icon(&app_title_icon, &current_app_info.borrow().app_id);

        let app_title_label = Label::builder()
            .label(&initial_name)
            .css_classes(["app-title"])
            .build();

        let app_title_content = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .valign(gtk4::Align::Center)
            .build();
        app_title_content.append(&app_title_icon);
        app_title_content.append(&app_title_label);
        app_title_btn.set_child(Some(&app_title_content));
        container.append(&app_title_btn);

        // 2b. Divider between the app title and its menu bar. Hidden until the
        // D-Bus menubar actually has entries, so no dangling "|" is rendered.
        let sep = VerticalSeparator::new();
        sep.widget().set_label("|");
        sep.widget().add_css_class("app-menu-sep");
        sep.widget().set_visible(false);
        container.append(sep.widget());

        // 3. Dynamic Application Title Context Menu (Real Window Operations)
        let app_info_click = current_app_info.clone();
        let title_btn_ref = app_title_btn.clone();
        let title_context_click = GestureClick::new();
        title_context_click.set_button(3);
        title_context_click.connect_pressed(move |_, _, _, _| {
            let info = app_info_click.borrow().clone();
            show_window_action_menu(&title_btn_ref, &info);
        });
        app_title_btn.add_controller(title_context_click);

        // 4. Dynamic D-Bus Menubar Box (populated strictly via D-Bus GetLayout)
        let menubar_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(2)
            .valign(gtk4::Align::Center)
            .build();
        container.append(&menubar_box);

        // GTK4 has no child-added/child-removed signals on Box, so keep the
        // separator in sync with the asynchronously populated menubar cheaply.
        let sep_sync = sep.widget().clone();
        let menubar_sync = menubar_box.clone();
        glib::timeout_add_local(Duration::from_millis(50), move || {
            sep_sync.set_visible(menubar_sync.first_child().is_some());
            glib::ControlFlow::Continue
        });

        // 5. Setup periodic polling for real-time active window detection
        let container_poll = container.clone();
        let app_info_poll = current_app_info.clone();
        let icon_poll = app_title_icon.clone();
        let label_poll = app_title_label.clone();
        let menubar_poll = menubar_box.clone();
        let service_poll = current_service.clone();
        let path_poll = current_menu_path.clone();
        let gen_poll = generation.clone();
        let last_discovery = Rc::new(RefCell::new(None::<Instant>));
        let last_discovery_poll = last_discovery.clone();

        // Initial D-Bus menu query if active window exists
        {
            let info = app_info_poll.borrow().clone();
            if !info.is_empty() {
                trigger_dbus_menu_discovery(
                    &info.app_id,
                    info.process_id,
                    0,
                    gen_poll.clone(),
                    menubar_poll.clone(),
                    service_poll.clone(),
                    path_poll.clone(),
                );
                *last_discovery.borrow_mut() = Some(Instant::now());
            }
        }

        glib::timeout_add_local(Duration::from_millis(150), move || {
            let latest = get_active_window().unwrap_or_default();
            let changed = {
                let cur = app_info_poll.borrow();
                cur.app_id != latest.app_id
                    || cur.title != latest.title
                    || cur.process_id != latest.process_id
            };

            let has_app = !latest.is_empty();
            let discovery_retry_due = has_app
                && !changed
                && service_poll.borrow().is_none()
                && last_discovery_poll
                    .borrow()
                    .is_none_or(|attempt| attempt.elapsed() >= Duration::from_secs(1));

            match true {
                _ if changed => {
                    *app_info_poll.borrow_mut() = latest.clone();
                    container_poll.set_visible(has_app);

                    let next_gen = *gen_poll.borrow() + 1;
                    *gen_poll.borrow_mut() = next_gen;
                    clear_menubar(&menubar_poll, &service_poll, &path_poll);

                    if has_app {
                        let display_name = get_app_display_name(&latest.app_id, &latest.title);
                        update_app_icon(&icon_poll, &latest.app_id);
                        label_poll.set_label(&display_name);
                        trigger_dbus_menu_discovery(
                            &latest.app_id,
                            latest.process_id,
                            next_gen,
                            gen_poll.clone(),
                            menubar_poll.clone(),
                            service_poll.clone(),
                            path_poll.clone(),
                        );
                        *last_discovery_poll.borrow_mut() = Some(Instant::now());
                    } else {
                        label_poll.set_label("");
                        *last_discovery_poll.borrow_mut() = None;
                    }
                }
                _ if discovery_retry_due => {
                    let next_gen = *gen_poll.borrow() + 1;
                    *gen_poll.borrow_mut() = next_gen;
                    trigger_dbus_menu_discovery(
                        &latest.app_id,
                        latest.process_id,
                        next_gen,
                        gen_poll.clone(),
                        menubar_poll.clone(),
                        service_poll.clone(),
                        path_poll.clone(),
                    );
                    *last_discovery_poll.borrow_mut() = Some(Instant::now());
                }
                _ => {}
            }

            glib::ControlFlow::Continue
        });

        Self {
            container,
            sep,
            menubar_box,
            app_title_icon,
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
            update_app_icon(&self.app_title_icon, app_name);
            self.app_title_label.set_label(app_name);
            *self.current_app_info.borrow_mut() = ActiveWindowInfo {
                app_id: app_name.to_string(),
                title: app_name.to_string(),
                process_id: None,
            };
        } else {
            self.app_title_label.set_label("");
            *self.current_app_info.borrow_mut() = ActiveWindowInfo::default();
            self.clear_menu();
        }
        let next_gen = *self.generation.borrow() + 1;
        *self.generation.borrow_mut() = next_gen;
        self.sync_separator();
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
        self.sync_separator();
    }

    /// Clears any dynamic D-Bus menubar buttons
    pub fn clear_menu(&self) {
        clear_menubar(
            &self.menubar_box,
            &self.current_service,
            &self.current_menu_path,
        );
        self.sync_separator();
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Only show the divider when the D-Bus menubar actually has entries.
    fn sync_separator(&self) {
        self.sep
            .widget()
            .set_visible(self.menubar_box.first_child().is_some());
    }
}

fn update_app_icon(icon: &gtk4::Image, app_id: &str) {
    // AppInfo/GIcon resolution is centralized in ui-kit; the GTK Image uses
    // the gio::Icon directly so the icon theme stays live.
    if let Some(gicon) = ui_kit::resolve_app_gicon(app_id) {
        icon.set_from_gicon(&gicon);
        icon.set_pixel_size(16);
        return;
    }
    icon.set_icon_name(Some("application-x-executable"));
    icon.set_pixel_size(16);
}

impl Default for AppMenu {
    fn default() -> Self {
        Self::new()
    }
}
