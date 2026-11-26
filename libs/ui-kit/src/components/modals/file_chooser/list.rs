//! File list rendering and row interaction for the file chooser.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box, GestureClick, Label, ListBox, ListBoxRow, Orientation, Window};
use services::filedialog::FileDialogMode;

use crate::icons::get_icon;
use crate::trans;

use super::format::{format_size, format_time};
use super::state::{visible_rows, CallbackSlot, DialogState, FileRow};

/// Creates the list renderer closure that repopulates rows from [`DialogState`].
pub(super) fn create_renderer(
    state: &Rc<RefCell<DialogState>>,
    list: &ListBox,
    navigate: &Rc<dyn Fn(PathBuf, bool)>,
    window: &Window,
    callback: &CallbackSlot,
) -> Rc<dyn Fn()> {
    let state = state.clone();
    let list = list.clone();
    let navigate = navigate.clone();
    let window = window.clone();
    let callback = callback.clone();

    Rc::new(move || {
        while let Some(child) = list.first_child() {
            list.remove(&child);
        }
        let visible = visible_rows(&state);
        if visible.is_empty() {
            let row = ListBoxRow::new();
            row.set_selectable(false);
            row.set_activatable(false);
            let label = Label::new(Some(&trans("file_dialog.empty_folder")));
            label.add_css_class("file-chooser-empty");
            label.set_margin_top(32);
            label.set_margin_bottom(32);
            row.set_child(Some(&label));
            list.append(&row);
        }
        let confirm_on_double_click = state.borrow().options.mode == FileDialogMode::OpenFile;
        for entry in visible {
            list.append(&build_file_row(
                &entry,
                &navigate,
                &window,
                &callback,
                confirm_on_double_click,
            ));
        }
    })
}

/// Builds one selectable file row.
fn build_file_row(
    entry: &FileRow,
    navigate: &Rc<dyn Fn(PathBuf, bool)>,
    window: &Window,
    callback: &CallbackSlot,
    confirm_on_double_click: bool,
) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.add_css_class("file-chooser-row");

    let box_row = Box::new(Orientation::Horizontal, 12);
    box_row.set_margin_start(10);
    box_row.set_margin_end(10);
    box_row.set_margin_top(6);
    box_row.set_margin_bottom(6);
    box_row.append(&get_icon(entry.icon, 22));

    let name = Label::new(Some(&entry.name));
    name.add_css_class("file-chooser-filename");
    name.set_halign(Align::Start);
    name.set_hexpand(true);
    box_row.append(&name);

    let size_text = if entry.is_dir {
        String::new()
    } else {
        format_size(entry.size)
    };
    let size = Label::new(Some(&size_text));
    size.add_css_class("file-chooser-size");
    size.set_width_request(80);
    size.set_halign(Align::End);
    box_row.append(&size);

    let date = Label::new(Some(&entry.modified.map(format_time).unwrap_or_default()));
    date.add_css_class("file-chooser-date");
    date.set_width_request(130);
    date.set_halign(Align::End);
    box_row.append(&date);
    row.set_child(Some(&box_row));

    let gesture = GestureClick::new();
    let path = entry.path.clone();
    let is_dir = entry.is_dir;
    let navigate = navigate.clone();
    let window = window.clone();
    let callback = callback.clone();
    gesture.connect_pressed(move |_, presses, _, _| {
        if presses != 2 {
            return;
        }
        if is_dir {
            navigate(path.clone(), true);
        } else if confirm_on_double_click {
            if let Some(done) = callback.borrow_mut().take() {
                done(Some(vec![path.clone()]));
                window.close();
            }
        }
    });
    row.add_controller(gesture);
    row
}
