//! Native BabyDra file chooser window composition and signal wiring.

use std::boxed::Box as StdBox;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::gdk::Key;
use gtk4::prelude::*;
use gtk4::{
    Application, Box, Button, EventControllerKey, Label, ListBox, Orientation, PolicyType,
    ScrolledWindow, SelectionMode, Window,
};
use services::filedialog::{FileDialogMode, FileDialogOptions};

use crate::trans;

use super::format::breadcrumb_segments;
use super::list::create_renderer;
use super::sidebar::build_sidebar;
use super::state::{
    confirm_paths, load_rows, CallbackSlot, DialogState, NavigateSlot, Navigator, RenderSlot,
};
use super::toolbar::{build_bottom_bar, build_header, build_nav_row};

/// Presents the chooser and reports the selection once.
///
/// # Arguments
/// * `parent` - Optional transient parent window.
/// * `options` - Initial folder, mode, title and filters.
/// * `on_selected` - Receives selected paths, or `None` when cancelled.
pub fn show(
    parent: Option<&Window>,
    options: FileDialogOptions,
    on_selected: impl Fn(Option<Vec<PathBuf>>) + 'static,
) {
    let initial = options
        .initial_folder
        .clone()
        .filter(|path| path.is_dir())
        .unwrap_or_else(gtk4::glib::home_dir);

    let state = Rc::new(RefCell::new(DialogState {
        filter_index: options.default_filter_index,
        show_hidden: options.show_hidden,
        options,
        current: initial,
        back: Vec::new(),
        forward: Vec::new(),
        selected: Vec::new(),
        rows: Vec::new(),
        displayed: Vec::new(),
        query: String::new(),
    }));
    let callback: CallbackSlot = Rc::new(RefCell::new(Some(StdBox::new(on_selected))));

    let window = Window::new();
    if let Some(app) = gtk4::gio::Application::default()
        .and_then(|app| app.downcast::<Application>().ok())
    {
        window.set_application(Some(&app));
    }
    window.set_title(Some(&state.borrow().options.title));
    window.set_default_size(880, 560);
    window.add_css_class("file-chooser-window");
    window.add_css_class("explore-dialog");
    if let Some(parent) = parent {
        window.set_transient_for(Some(parent));
        window.set_modal(true);
    }

    let root = Box::new(Orientation::Vertical, 0);
    window.set_child(Some(&root));

    let header = build_header(&state.borrow().options);
    let nav = build_nav_row();
    let (sidebar_scroll, sidebar_buttons) = build_sidebar();
    let bottom = build_bottom_bar(&state.borrow().options, state.borrow().filter_index);

    let content_scroll = ScrolledWindow::new();
    content_scroll.set_policy(PolicyType::Automatic, PolicyType::Automatic);
    content_scroll.add_css_class("file-chooser-content-scroll");
    content_scroll.set_hexpand(true);
    content_scroll.set_vexpand(true);
    let list = ListBox::new();
    list.add_css_class("file-chooser-list");
    list.set_selection_mode(match state.borrow().options.mode {
        FileDialogMode::OpenMultipleFiles => SelectionMode::Multiple,
        _ => SelectionMode::Single,
    });
    content_scroll.set_child(Some(&list));

    let split = Box::new(Orientation::Horizontal, 0);
    split.set_vexpand(true);
    split.set_hexpand(true);
    split.append(&sidebar_scroll);
    split.append(&content_scroll);

    root.append(&header.root);
    root.append(&nav.root);
    root.append(&split);
    root.append(&bottom.root);

    let render: RenderSlot = Rc::new(RefCell::new(None));
    let nav_slot: NavigateSlot = Rc::new(RefCell::new(None));
    let navigate_to = create_navigator(
        &state,
        &render,
        &nav_slot,
        &sidebar_buttons,
        &nav.breadcrumbs,
        &nav.btn_back,
        &nav.btn_forward,
        &nav.path_entry,
    );
    *nav_slot.borrow_mut() = Some(navigate_to.clone());

    for (path, button) in sidebar_buttons.borrow().iter() {
        let path = path.clone();
        let navigate = navigate_to.clone();
        button.connect_clicked(move |_| navigate(path.clone(), true));
    }

    let renderer = create_renderer(&state, &list, &navigate_to, &window, &callback);
    *render.borrow_mut() = Some(renderer);

    wire_list_selection(&list, &state, &bottom.name_entry);
    wire_confirm_cancel(&window, &state, &callback, &header, &bottom, &nav.path_entry, &navigate_to);
    wire_toolbar(&window, &state, &render, &navigate_to, &nav, &bottom.filter_dropdown);

    let initial = state.borrow().current.clone();
    navigate_to(initial, false);
    window.present();
}

#[allow(clippy::too_many_arguments)]
fn create_navigator(
    state: &Rc<RefCell<DialogState>>,
    render: &RenderSlot,
    nav_slot: &NavigateSlot,
    sidebar_buttons: &Rc<RefCell<Vec<(PathBuf, Button)>>>,
    breadcrumbs: &Box,
    btn_back: &Button,
    btn_forward: &Button,
    path_entry: &gtk4::Entry,
) -> Navigator {
    let state = state.clone();
    let render = render.clone();
    let nav_slot = nav_slot.clone();
    let sidebar_buttons = sidebar_buttons.clone();
    let breadcrumbs = breadcrumbs.clone();
    let btn_back = btn_back.clone();
    let btn_forward = btn_forward.clone();
    let path_entry = path_entry.clone();

    Rc::new(move |target: PathBuf, record_history: bool| {
        if !target.is_dir() {
            return;
        }
        if record_history {
            let previous = state.borrow().current.clone();
            if previous != target {
                let mut borrowed = state.borrow_mut();
                borrowed.back.push(previous);
                borrowed.forward.clear();
            }
        }
        {
            let mut borrowed = state.borrow_mut();
            borrowed.current = target.clone();
            borrowed.selected.clear();
            borrowed.rows = load_rows(&target);
        }
        path_entry.set_text(&target.to_string_lossy());
        btn_back.set_sensitive(!state.borrow().back.is_empty());
        btn_forward.set_sensitive(!state.borrow().forward.is_empty());

        for (path, button) in sidebar_buttons.borrow().iter() {
            if *path == target {
                button.add_css_class("active-nav");
            } else {
                button.remove_css_class("active-nav");
            }
        }

        while let Some(child) = breadcrumbs.first_child() {
            breadcrumbs.remove(&child);
        }
        for (index, (name, path)) in breadcrumb_segments(&target).into_iter().enumerate() {
            if index > 0 {
                let sep = Label::new(Some("›"));
                sep.add_css_class("file-chooser-breadcrumb-sep");
                breadcrumbs.append(&sep);
            }
            let button = Button::with_label(&name);
            button.add_css_class("file-chooser-breadcrumb-btn");
            button.add_css_class("flat");
            button.set_cursor_from_name(Some("pointer"));
            let nav_slot = nav_slot.clone();
            button.connect_clicked(move |_| {
                if let Some(navigate) = nav_slot.borrow().as_ref() {
                    navigate(path.clone(), true);
                }
            });
            breadcrumbs.append(&button);
        }

        if let Some(renderer) = render.borrow().as_ref() {
            renderer();
        }
    })
}

fn wire_list_selection(
    list: &ListBox,
    state: &Rc<RefCell<DialogState>>,
    name_entry: &gtk4::Entry,
) {
    let state = state.clone();
    let name_entry = name_entry.clone();
    list.connect_row_selected(move |list, _| {
        let selected: Vec<PathBuf> = list
            .selected_rows()
            .iter()
            .filter_map(|row| {
                state
                    .borrow()
                    .displayed
                    .get(row.index() as usize)
                    .map(|entry| entry.path.clone())
            })
            .collect();
        state.borrow_mut().selected = selected.clone();
        if let Some(name) = selected.first().and_then(|path| path.file_name()) {
            name_entry.set_text(&name.to_string_lossy());
        }
    });
}

fn wire_confirm_cancel(
    window: &Window,
    state: &Rc<RefCell<DialogState>>,
    callback: &CallbackSlot,
    header: &super::toolbar::HeaderWidgets,
    bottom: &super::toolbar::BottomWidgets,
    path_entry: &gtk4::Entry,
    navigate_to: &Navigator,
) {
    let do_confirm: Rc<dyn Fn()> = {
        let state = state.clone();
        let name_entry = bottom.name_entry.clone();
        let callback = callback.clone();
        let window = window.clone();
        Rc::new(move || {
            let typed = name_entry.text().trim().to_string();
            let paths = confirm_paths(&state, &typed);
            if paths.is_empty() {
                return;
            }
            if let Some(done) = callback.borrow_mut().take() {
                done(Some(paths));
            }
            window.close();
        })
    };
    let cancel_once: Rc<dyn Fn()> = {
        let callback = callback.clone();
        Rc::new(move || {
            if let Some(done) = callback.borrow_mut().take() {
                done(None);
            }
        })
    };
    let do_cancel: Rc<dyn Fn()> = {
        let cancel_once = cancel_once.clone();
        let window = window.clone();
        Rc::new(move || {
            cancel_once();
            window.close();
        })
    };

    for button in [&header.btn_confirm, &bottom.btn_confirm] {
        let confirm = do_confirm.clone();
        button.connect_clicked(move |_| confirm());
    }
    for button in [&header.btn_cancel, &bottom.btn_cancel] {
        let cancel = do_cancel.clone();
        button.connect_clicked(move |_| cancel());
    }
    let confirm = do_confirm.clone();
    bottom.name_entry.connect_activate(move |_| confirm());

    let navigate = navigate_to.clone();
    path_entry.connect_activate(move |entry| {
        navigate(PathBuf::from(entry.text().to_string()), true);
    });

    let keys = EventControllerKey::new();
    let key_cancel = do_cancel;
    keys.connect_key_pressed(move |_, key, _, _| {
        if key == Key::Escape {
            key_cancel();
            return gtk4::glib::Propagation::Stop;
        }
        gtk4::glib::Propagation::Proceed
    });
    window.add_controller(keys);

    window.connect_close_request(move |_| {
        cancel_once();
        gtk4::glib::Propagation::Proceed
    });
}

fn wire_toolbar(
    window: &Window,
    state: &Rc<RefCell<DialogState>>,
    render: &RenderSlot,
    navigate_to: &Navigator,
    nav: &super::toolbar::NavWidgets,
    filter_dropdown: &gtk4::DropDown,
) {
    {
        let state = state.clone();
        let navigate = navigate_to.clone();
        nav.btn_back.connect_clicked(move |_| {
            let Some(prev) = state.borrow_mut().back.pop() else {
                return;
            };
            let current = state.borrow().current.clone();
            state.borrow_mut().forward.push(current);
            navigate(prev, false);
        });
    }
    {
        let state = state.clone();
        let navigate = navigate_to.clone();
        nav.btn_forward.connect_clicked(move |_| {
            let Some(next) = state.borrow_mut().forward.pop() else {
                return;
            };
            let current = state.borrow().current.clone();
            state.borrow_mut().back.push(current);
            navigate(next, false);
        });
    }
    {
        let state = state.clone();
        let navigate = navigate_to.clone();
        nav.btn_up.connect_clicked(move |_| {
            if let Some(parent) = state.borrow().current.parent() {
                navigate(parent.to_path_buf(), true);
            }
        });
    }
    {
        let state = state.clone();
        let render = render.clone();
        nav.search_entry.connect_changed(move |entry| {
            state.borrow_mut().query = entry.text().to_string();
            if let Some(renderer) = render.borrow().as_ref() {
                renderer();
            }
        });
    }
    {
        let state = state.clone();
        let render = render.clone();
        nav.btn_hidden.connect_clicked(move |_| {
            let current = state.borrow().show_hidden;
            state.borrow_mut().show_hidden = !current;
            if let Some(renderer) = render.borrow().as_ref() {
                renderer();
            }
        });
    }
    {
        let state = state.clone();
        let render = render.clone();
        let keys = EventControllerKey::new();
        keys.connect_key_pressed(move |_, key, _, modifier| {
            if modifier.contains(gtk4::gdk::ModifierType::CONTROL_MASK)
                && (key == Key::h || key == Key::H)
            {
                let current = state.borrow().show_hidden;
                state.borrow_mut().show_hidden = !current;
                if let Some(renderer) = render.borrow().as_ref() {
                    renderer();
                }
                return gtk4::glib::Propagation::Stop;
            }
            gtk4::glib::Propagation::Proceed
        });
        window.add_controller(keys);
    }
    {
        let state = state.clone();
        let navigate = navigate_to.clone();
        nav.btn_new_folder.connect_clicked(move |_| {
            let directory = state.borrow().current.clone();
            let base = trans("file_dialog.new_folder_title");
            let mut name = base.clone();
            let mut count = 1;
            while directory.join(&name).exists() {
                count += 1;
                name = format!("{base} {count}");
            }
            if std::fs::create_dir(directory.join(&name)).is_ok() {
                navigate(directory.join(&name), true);
            }
        });
    }
    {
        let state = state.clone();
        let render = render.clone();
        filter_dropdown.connect_selected_notify(move |dropdown| {
            state.borrow_mut().filter_index = Some(dropdown.selected() as usize);
            if let Some(renderer) = render.borrow().as_ref() {
                renderer();
            }
        });
    }
}
