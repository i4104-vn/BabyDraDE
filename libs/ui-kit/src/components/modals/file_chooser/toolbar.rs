//! Header, navigation row and bottom action bar builders for the file chooser.

use gtk4::prelude::*;
use gtk4::{Align, Box, Button, DropDown, Entry, Label, Orientation, StringList};
use services::filedialog::{FileDialogMode, FileDialogOptions};

use crate::icons::get_icon;
use crate::trans;

use super::state::confirm_label;

pub(super) struct HeaderWidgets {
    pub root: Box,
    pub btn_cancel: Button,
    pub btn_confirm: Button,
}

pub(super) struct NavWidgets {
    pub root: Box,
    pub btn_back: Button,
    pub btn_forward: Button,
    pub btn_up: Button,
    pub breadcrumbs: Box,
    pub path_entry: Entry,
    pub search_entry: Entry,
    pub btn_hidden: Button,
    pub btn_new_folder: Button,
}

pub(super) struct BottomWidgets {
    pub root: Box,
    pub name_entry: Entry,
    pub filter_dropdown: DropDown,
    pub btn_cancel: Button,
    pub btn_confirm: Button,
}

/// Builds the title and top action buttons.
pub(super) fn build_header(options: &FileDialogOptions) -> HeaderWidgets {
    let root = Box::new(Orientation::Horizontal, 12);
    root.add_css_class("file-chooser-header");
    root.set_margin_start(16);
    root.set_margin_end(16);
    root.set_margin_top(12);
    root.set_margin_bottom(8);

    let title = Label::new(Some(&options.title));
    title.add_css_class("file-chooser-title");
    title.set_halign(Align::Start);
    title.set_hexpand(true);
    root.append(&title);

    let btn_cancel = Button::with_label(&trans("file_dialog.btn_cancel"));
    btn_cancel.add_css_class("modern-dialog-cancel-btn");
    btn_cancel.set_cursor_from_name(Some("pointer"));
    root.append(&btn_cancel);

    let btn_confirm = Button::with_label(&confirm_label(options));
    btn_confirm.add_css_class("modern-dialog-primary-btn");
    btn_confirm.set_cursor_from_name(Some("pointer"));
    root.append(&btn_confirm);

    HeaderWidgets {
        root,
        btn_cancel,
        btn_confirm,
    }
}

/// Builds the navigation toolbar with history, breadcrumbs, path, search and actions.
pub(super) fn build_nav_row() -> NavWidgets {
    let root = Box::new(Orientation::Horizontal, 6);
    root.add_css_class("file-chooser-nav-row");
    root.set_margin_start(16);
    root.set_margin_end(16);
    root.set_margin_bottom(8);

    let btn_back = nav_button("arrow-left", "file_dialog.tooltip_back");
    let btn_forward = nav_button("arrow-right", "file_dialog.tooltip_forward");
    let btn_up = nav_button("arrow-up", "file_dialog.tooltip_up");
    root.append(&btn_back);
    root.append(&btn_forward);
    root.append(&btn_up);

    let breadcrumbs = Box::new(Orientation::Horizontal, 2);
    breadcrumbs.add_css_class("file-chooser-breadcrumbs");
    breadcrumbs.set_hexpand(true);
    breadcrumbs.set_halign(Align::Fill);
    root.append(&breadcrumbs);

    let path_entry = Entry::new();
    path_entry.add_css_class("file-chooser-path-entry");
    path_entry.set_width_request(200);
    root.append(&path_entry);

    let search_entry = Entry::new();
    search_entry.set_placeholder_text(Some(&trans("file_dialog.search_placeholder")));
    search_entry.add_css_class("file-chooser-search");
    search_entry.set_width_request(140);
    root.append(&search_entry);

    let btn_hidden = nav_button("eye", "file_dialog.tooltip_toggle_hidden");
    let btn_new_folder = nav_button("folder-new", "file_dialog.tooltip_new_folder");
    root.append(&btn_hidden);
    root.append(&btn_new_folder);

    NavWidgets {
        root,
        btn_back,
        btn_forward,
        btn_up,
        breadcrumbs,
        path_entry,
        search_entry,
        btn_hidden,
        btn_new_folder,
    }
}

/// Builds the filename entry, filter dropdown and bottom action buttons.
pub(super) fn build_bottom_bar(
    options: &FileDialogOptions,
    filter_index: Option<usize>,
) -> BottomWidgets {
    let root = Box::new(Orientation::Horizontal, 10);
    root.add_css_class("file-chooser-bottom-bar");
    root.set_margin_start(16);
    root.set_margin_end(16);
    root.set_margin_top(10);
    root.set_margin_bottom(12);

    let name_label_text = if options.mode == FileDialogMode::OpenFolder {
        trans("file_dialog.title_open_folder")
    } else {
        trans("file_dialog.lbl_file_name")
    };
    let name_label = Label::new(Some(&name_label_text));
    name_label.add_css_class("file-chooser-bottom-lbl");
    root.append(&name_label);

    let name_entry = Entry::new();
    name_entry.set_hexpand(true);
    name_entry.add_css_class("file-chooser-name-entry");
    if let Some(name) = &options.initial_file {
        name_entry.set_text(name);
    }
    root.append(&name_entry);

    let filter_list = StringList::new(&[]);
    for filter in &options.filters {
        filter_list.append(&filter.name);
    }
    let filter_dropdown = DropDown::new(Some(filter_list), None::<gtk4::Expression>);
    filter_dropdown.add_css_class("file-chooser-filter-dropdown");
    if let Some(index) = filter_index {
        filter_dropdown.set_selected(index as u32);
    }
    if !options.filters.is_empty() {
        root.append(&filter_dropdown);
    }

    let btn_cancel = Button::with_label(&trans("file_dialog.btn_cancel"));
    btn_cancel.add_css_class("modern-dialog-cancel-btn");
    btn_cancel.set_cursor_from_name(Some("pointer"));
    root.append(&btn_cancel);

    let btn_confirm = Button::with_label(&confirm_label(options));
    btn_confirm.add_css_class("modern-dialog-primary-btn");
    btn_confirm.set_cursor_from_name(Some("pointer"));
    root.append(&btn_confirm);

    BottomWidgets {
        root,
        name_entry,
        filter_dropdown,
        btn_cancel,
        btn_confirm,
    }
}

fn nav_button(icon: &str, tooltip_key: &str) -> Button {
    let button = Button::new();
    button.set_child(Some(&get_icon(icon, 16)));
    button.add_css_class("flat");
    button.add_css_class("nav-btn");
    button.set_tooltip_text(Some(&trans(tooltip_key)));
    button
}
