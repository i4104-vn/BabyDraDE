//! Dynamic Island Visual Capsule Component

use gtk4::prelude::*;
use gtk4::{Box, GestureClick, Label, Orientation, Popover};
use std::cell::RefCell;
use std::rc::Rc;

use super::manager::IslandManager;

pub struct IslandCapsule {
    wrapper: Box,
    capsule_box: Box,
    content_slot: Box,
    popover: Popover,
    manager: Rc<RefCell<IslandManager>>,
}

impl IslandCapsule {
    pub fn new() -> Self {
        let manager = Rc::new(RefCell::new(IslandManager::new()));

        let wrapper = Box::builder()
            .orientation(Orientation::Horizontal)
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .css_classes(["island-wrapper"])
            .build();

        // Left bracket indicator "("
        let left_bracket = Label::builder()
            .label("(")
            .css_classes(["island-bracket", "left"])
            .build();
        wrapper.append(&left_bracket);

        // Center pill notch capsule
        let capsule_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .css_classes(["panel-notch", "active-music"])
            .build();

        if let Some(cursor) = gtk4::gdk::Cursor::from_name("pointer", None) {
            capsule_box.set_cursor(Some(&cursor));
        }

        let content_slot = Box::builder()
            .orientation(Orientation::Horizontal)
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .build();
        capsule_box.append(&content_slot);

        wrapper.append(&capsule_box);

        // Right bracket indicator ")"
        let right_bracket = Label::builder()
            .label(")")
            .css_classes(["island-bracket", "right"])
            .build();
        wrapper.append(&right_bracket);

        // Details Popover
        let popover = Popover::builder()
            .autohide(true)
            .has_arrow(false)
            .css_classes(["media-popover", "island-popover"])
            .position(gtk4::PositionType::Bottom)
            .build();
        popover.set_parent(&capsule_box);

        let s = Self {
            wrapper,
            capsule_box,
            content_slot,
            popover,
            manager,
        };

        s.refresh_active_view();

        // Click gesture on capsule to open details popover
        let popover_clone = s.popover.clone();
        let mgr_click = s.manager.clone();
        let click = GestureClick::new();
        click.connect_pressed(move |_, _, _, _| {
            let mgr = mgr_click.borrow();
            if let Some(feature) = mgr.active_feature() {
                if let Some(content) = feature.details_popover_content() {
                    popover_clone.set_child(Some(&content));
                    popover_clone.popup();
                }
            }
        });
        s.capsule_box.add_controller(click);

        s
    }

    /// Updates the slot widget to the active feature
    pub fn refresh_active_view(&self) {
        // Clear previous content
        while let Some(child) = self.content_slot.first_child() {
            self.content_slot.remove(&child);
        }

        let mgr = self.manager.borrow();
        if let Some(active) = mgr.active_feature() {
            self.content_slot.append(&active.widget());
        }
    }

    pub fn widget(&self) -> &Box {
        &self.wrapper
    }

    pub fn manager(&self) -> Rc<RefCell<IslandManager>> {
        self.manager.clone()
    }
}

impl Default for IslandCapsule {
    fn default() -> Self {
        Self::new()
    }
}
