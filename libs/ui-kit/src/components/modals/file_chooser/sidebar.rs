//! Quick access places sidebar for the file chooser.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Orientation, PolicyType, ScrolledWindow};
use services::filedialog::places::default_places;

use crate::icons::get_icon;
use crate::trans;

/// Builds the places sidebar scroll container and its `(path, button)` pairs.
pub(super) fn build_sidebar() -> (ScrolledWindow, Rc<RefCell<Vec<(PathBuf, Button)>>>) {
    let scroll = ScrolledWindow::new();
    scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
    scroll.add_css_class("file-chooser-sidebar-scroll");
    scroll.set_width_request(180);

    let sidebar = Box::new(Orientation::Vertical, 2);
    sidebar.add_css_class("file-chooser-sidebar");
    sidebar.set_margin_start(12);
    sidebar.set_margin_end(8);
    sidebar.set_margin_top(4);
    sidebar.set_margin_bottom(4);
    scroll.set_child(Some(&sidebar));

    let places_label = Label::new(Some(&trans("file_dialog.places_title")));
    places_label.add_css_class("file-chooser-sidebar-header");
    places_label.set_halign(Align::Start);
    places_label.set_margin_top(4);
    places_label.set_margin_bottom(4);
    sidebar.append(&places_label);

    let buttons = Rc::new(RefCell::new(Vec::new()));
    for place in default_places() {
        let button = Button::new();
        button.add_css_class("file-chooser-sidebar-btn");
        button.add_css_class("flat");
        button.set_cursor_from_name(Some("pointer"));
        let row = Box::new(Orientation::Horizontal, 8);
        row.set_halign(Align::Start);
        row.append(&get_icon(&place.icon_name, 18));
        let label = Label::new(Some(&place.name));
        label.add_css_class("file-chooser-sidebar-label");
        row.append(&label);
        button.set_child(Some(&row));
        sidebar.append(&button);
        buttons.borrow_mut().push((place.path, button));
    }

    (scroll, buttons)
}
