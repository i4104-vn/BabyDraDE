//! Launcher overlay: app list, live search, keyboard navigation and slide in/out.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use animation::{slide_in, slide_out_cb, SlideDirection};
use gtk4::gio::{prelude::AppInfoExt, AppInfo};
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use services::trans;

#[derive(Clone)]
struct AppEntry {
    info: AppInfo,
    name: String,
    desc: String,
}

/// Collects visible desktop applications via GIO and sorts them by name.
fn collect_apps() -> Vec<AppEntry> {
    let mut apps: Vec<AppEntry> = AppInfo::all()
        .into_iter()
        .filter(|info| info.should_show())
        .map(|info| {
            let name = info.display_name().to_string();
            let desc = info
                .description()
                .map(|text| text.to_string())
                .unwrap_or_default();
            AppEntry { info, name, desc }
        })
        .collect();
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

fn app_button(app: &AppEntry, window: &gtk4::ApplicationWindow) -> gtk4::Button {
    let button = gtk4::Button::new();
    button.add_css_class("launcher-list-item");

    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    if let Some(icon) = app.info.icon() {
        let image = gtk4::Image::from_gicon(&icon);
        image.set_pixel_size(22);
        let icon_wrap = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        icon_wrap.add_css_class("app-icon-wrapper");
        icon_wrap.append(&image);
        row.append(&icon_wrap);
    }

    let texts = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    let title = gtk4::Label::builder()
        .label(&app.name)
        .css_classes(["app-title"])
        .xalign(0.0)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .build();
    texts.append(&title);
    if !app.desc.is_empty() {
        let desc = gtk4::Label::builder()
            .label(&app.desc)
            .css_classes(["app-desc"])
            .xalign(0.0)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();
        texts.append(&desc);
    }
    row.append(&texts);
    button.set_child(Some(&row));

    let info = app.info.clone();
    let win = window.clone();
    button.connect_clicked(move |_| {
        let _ = info.launch(&[], None::<&gtk4::gio::AppLaunchContext>);
        win.close();
    });

    button
}

fn visible_buttons(list_box: &gtk4::Box) -> Vec<gtk4::Button> {
    let mut buttons = Vec::new();
    let mut child = list_box.first_child();
    while let Some(widget) = child {
        if let Some(button) = widget.downcast_ref::<gtk4::Button>() {
            buttons.push(button.clone());
        }
        child = widget.next_sibling();
    }
    buttons
}

fn highlight(list_box: &gtk4::Box, selected: usize) {
    for (index, button) in visible_buttons(list_box).iter().enumerate() {
        if index == selected {
            button.add_css_class("selected");
        } else {
            button.remove_css_class("selected");
        }
    }
}

fn repopulate(
    list_box: &gtk4::Box,
    apps: &[AppEntry],
    query: &str,
    selected: usize,
    window: &gtk4::ApplicationWindow,
) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    let needle = query.trim().to_lowercase();
    let matches: Vec<&AppEntry> = apps
        .iter()
        .filter(|app| {
            needle.is_empty()
                || app.name.to_lowercase().contains(&needle)
                || app.desc.to_lowercase().contains(&needle)
                || app.info.executable().to_string_lossy().to_lowercase().contains(&needle)
        })
        .collect();

    if matches.is_empty() {
        let empty = gtk4::Label::builder()
            .label(trans("launcher.no_results"))
            .css_classes(["status-msg"])
            .build();
        list_box.append(&empty);
        return;
    }

    for app in matches {
        list_box.append(&app_button(app, window));
    }
    let selected = selected.min(visible_buttons(list_box).len().saturating_sub(1));
    highlight(list_box, selected);
}

/// Builds the launcher window. The shared slot lets `main.rs` toggle/hide it.
pub fn build_launcher_ui(
    app: &gtk4::Application,
    window_slot: Rc<RefCell<Option<gtk4::ApplicationWindow>>>,
) -> gtk4::ApplicationWindow {
    let window = gtk4::ApplicationWindow::new(app);
    window.add_css_class("launcher-window");
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    window.set_exclusive_zone(-1);
    for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
        window.set_anchor(edge, true);
    }

    let content = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(0)
        .css_classes(["launcher-box"])
        .width_request(420)
        .height_request(600)
        .margin_top(50)
        .margin_start(16)
        .halign(gtk4::Align::Start)
        .valign(gtk4::Align::Start)
        .build();
    window.set_child(Some(&content));

    let header = gtk4::Label::builder()
        .label(trans("launcher.apps"))
        .css_classes(["brand-text"])
        .xalign(0.0)
        .margin_top(20)
        .margin_start(24)
        .margin_end(24)
        .margin_bottom(12)
        .build();
    content.append(&header);

    let search = gtk4::Entry::builder()
        .placeholder_text(trans("launcher.search_placeholder"))
        .css_classes(["launcher-search"])
        .margin_start(24)
        .margin_end(24)
        .margin_bottom(12)
        .build();
    content.append(&search);

    let scrolled = gtk4::ScrolledWindow::builder()
        .css_classes(["list-container"])
        .hexpand(true)
        .vexpand(true)
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .margin_start(24)
        .margin_end(24)
        .build();
    let list_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    scrolled.set_child(Some(&list_box));
    content.append(&scrolled);

    let footer = gtk4::Label::builder()
        .label("↑ ↓  navigate   •   ↵  launch   •   Esc  close")
        .css_classes(["launcher-footer-box"])
        .margin_start(20)
        .margin_end(20)
        .margin_bottom(14)
        .build();
    content.append(&footer);

    let apps = Rc::new(collect_apps());
    let query = Rc::new(RefCell::new(String::new()));
    let selected = Rc::new(RefCell::new(0usize));
    let debounce: Rc<RefCell<Option<gtk4::glib::SourceId>>> = Rc::new(RefCell::new(None));
    let animating = Rc::new(Cell::new(false));

    repopulate(&list_box, &apps, "", 0, &window);

    // Live search with a short debounce.
    {
        let (_search_c, list_c, apps_c, query_c, sel_c, deb_c) = (
            search.clone(),
            list_box.clone(),
            apps.clone(),
            query.clone(),
            selected.clone(),
            debounce.clone(),
        );
        let window_c = window.clone();
        search.connect_changed(move |entry| {
            let text = entry.text().to_string();
            *query_c.borrow_mut() = text;
            if let Some(source) = deb_c.borrow_mut().take() {
                source.remove();
            }
            let (list, apps, query, sel, win, deb) = (
                list_c.clone(),
                apps_c.clone(),
                query_c.clone(),
                sel_c.clone(),
                window_c.clone(),
                deb_c.clone(),
            );
            let debounce_source = deb.clone();
            *deb.borrow_mut() = Some(gtk4::glib::timeout_add_local_once(
                std::time::Duration::from_millis(200),
                move || {
                    *debounce_source.borrow_mut() = None;
                    *sel.borrow_mut() = 0;
                    repopulate(&list, &apps, &query.borrow(), 0, &win);
                },
            ));
        });
    }

    // Keyboard navigation and type-to-search.
    let key = gtk4::EventControllerKey::new();
    key.set_propagation_phase(gtk4::PropagationPhase::Capture);
    {
        let (list, sel, search, win) = (list_box.clone(), selected.clone(), search.clone(), window.clone());
        key.connect_key_pressed(move |_, key, _, _| match key {
            gtk4::gdk::Key::Escape => {
                win.close();
                gtk4::glib::Propagation::Stop
            }
            gtk4::gdk::Key::Down | gtk4::gdk::Key::Up => {
                let buttons = visible_buttons(&list);
                if !buttons.is_empty() {
                    let current = *sel.borrow();
                    let next = if key == gtk4::gdk::Key::Down {
                        (current + 1) % buttons.len()
                    } else {
                        (current + buttons.len() - 1) % buttons.len()
                    };
                    *sel.borrow_mut() = next;
                    highlight(&list, next);
                    buttons[next].grab_focus();
                }
                gtk4::glib::Propagation::Stop
            }
            gtk4::gdk::Key::Return | gtk4::gdk::Key::KP_Enter => {
                let buttons = visible_buttons(&list);
                if let Some(button) = buttons.get(*sel.borrow()) {
                    button.activate();
                }
                gtk4::glib::Propagation::Stop
            }
            _ => {
                if !search.is_focus() {
                    if let Some(character) = key.to_unicode() {
                        if !character.is_control() {
                            let text = format!("{}{}", search.text(), character);
                            search.set_text(&text);
                            search.set_position(-1);
                            search.grab_focus();
                            return gtk4::glib::Propagation::Stop;
                        }
                    }
                }
                gtk4::glib::Propagation::Proceed
            }
        });
    }
    window.add_controller(key);

    // Click outside the panel dismisses the launcher.
    let click = gtk4::GestureClick::new();
    let (content_c, win_c) = (content.clone(), window.clone());
    click.connect_pressed(move |_, _, x, y| {
        let inside = win_c
            .pick(x, y, gtk4::PickFlags::DEFAULT)
            .is_some_and(|picked| picked == content_c || picked.is_ancestor(&content_c));
        if !inside {
            win_c.close();
        }
    });
    window.add_controller(click);

    let active_window = window.clone();
    window.connect_is_active_notify(move |win| {
        if !win.is_active() {
            let _ = &active_window;
            win.close();
        }
    });

    // Slide-out on close, then drop the shared slot so main.rs can reopen it.
    {
        let (content_close, slot_close, anim_close) =
            (content.clone(), window_slot.clone(), animating.clone());
        let win_close = window.clone();
        window.connect_close_request(move |_| {
            if anim_close.get() {
                return gtk4::glib::Propagation::Stop;
            }
            anim_close.set(true);
            let win_cb = win_close.clone();
            let slot_cb = slot_close.clone();
            slide_out_cb(
                content_close.upcast_ref(),
                SlideDirection::Up,
                32,
                180,
                false,
                move || {
                    win_cb.destroy();
                    if let Ok(mut slot) = slot_cb.try_borrow_mut() {
                        *slot = None;
                    }
                },
            );
            gtk4::glib::Propagation::Stop
        });
    }

    {
        let list_focus = list_box.clone();
        let sel_focus = selected.clone();
        window.connect_focus_widget_notify(move |win| {
            if let Some(focused) = gtk4::prelude::RootExt::focus(win) {
                if let Some(index) = visible_buttons(&list_focus)
                    .iter()
                    .position(|button| button.clone().upcast::<gtk4::Widget>() == focused)
                {
                    *sel_focus.borrow_mut() = index;
                    highlight(&list_focus, index);
                }
            }
        });
    }

    window.connect_map(move |_| {
        gtk4::glib::timeout_add_local_once(std::time::Duration::from_millis(50), {
            let search_map = search.clone();
            move || {
                search_map.grab_focus();
            }
        });
    });

    slide_in(content.upcast_ref(), SlideDirection::Down, 32, 180);
    window
}
