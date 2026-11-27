//! Builds the island widget tree (wrapper, brackets, notch capsule).

use std::cell::Cell;

use gtk4::prelude::*;

pub(crate) struct Shell {
    pub(crate) root: gtk4::Box,
    pub(crate) capsule: gtk4::Box,
    pub(crate) content: gtk4::Box,
    pub(crate) left_bracket: gtk4::Label,
    pub(crate) right_bracket: gtk4::Label,
    pub(crate) hovered: Cell<bool>,
}

pub(crate) fn build() -> Shell {
    let root = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .css_classes(["island-wrapper"])
        .build();
    let left_bracket = gtk4::Label::builder()
        .label("(")
        .css_classes(["island-bracket", "left"])
        .visible(false)
        .build();
    let capsule = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .css_classes(["panel-notch"])
        .visible(false)
        .build();
    if let Some(cursor) = gtk4::gdk::Cursor::from_name("pointer", None) {
        capsule.set_cursor(Some(&cursor));
    }
    let content = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .css_classes(["notch-content"])
        .build();
    capsule.append(&content);
    let right_bracket = gtk4::Label::builder()
        .label(")")
        .css_classes(["island-bracket", "right"])
        .visible(false)
        .build();
    root.append(&left_bracket);
    root.append(&capsule);
    root.append(&right_bracket);

    let hovered = Cell::new(false);
    let enter = gtk4::EventControllerMotion::new();
    let hover_enter = hovered.clone();
    enter.connect_enter(move |_, _, _| hover_enter.set(true));
    let leave = gtk4::EventControllerMotion::new();
    let hover_leave = hovered.clone();
    leave.connect_leave(move |_| hover_leave.set(false));
    capsule.add_controller(enter);
    capsule.add_controller(leave);

    Shell {
        root,
        capsule,
        content,
        left_bracket,
        right_bracket,
        hovered,
    }
}
