//! Fluent builder for declaratively constructing and presenting context menus.

use super::items::*;
use gtk4::prelude::*;
use gtk4::{Box, Button, Popover, PositionType};
use std::cell::RefCell;
use std::rc::Rc;

/// A Fluent Context Menu overlay component.
pub struct ContextMenu {
    popover: Popover,
    vbox: Box,
    footer_box: Option<Box>,
    active_subpopover: Rc<RefCell<Option<Popover>>>,
    ancestor_popovers: Vec<Popover>,
}

pub type ContextMenuBuilder = ContextMenu;

impl ContextMenu {
    /// Starts building a context menu attached to `parent`.
    pub fn new(parent: &impl IsA<gtk4::Widget>) -> Self {
        let popover = Popover::builder().has_arrow(false).autohide(true).build();
        popover.set_parent(parent.as_ref());
        popover.add_css_class("context-menu-popover");
        popover.add_css_class("explore-popover");
        popover.add_css_class("desktop-context-menu");

        let vbox = create_menu_box(200);
        popover.set_child(Some(&vbox));

        let active_subpopover: Rc<RefCell<Option<Popover>>> = Rc::new(RefCell::new(None));
        let active_sub_close = active_subpopover.clone();
        popover.connect_closed(move |_| {
            if let Some(sub) = active_sub_close.borrow_mut().take() {
                sub.popdown();
            }
        });

        Self {
            popover,
            vbox,
            footer_box: None,
            active_subpopover,
            ancestor_popovers: Vec::new(),
        }
    }

    /// Positions the popover at exact pointer `(x, y)` coordinates.
    pub fn at_coords(parent: &impl IsA<gtk4::Widget>, x: f64, y: f64) -> Self {
        let menu = Self::new(parent);
        menu.popover.set_has_arrow(false);
        let rect = gtk4::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
        menu.popover.set_pointing_to(Some(&rect));
        menu
    }

    /// Anchors the popover relative to the parent widget with an arrow.
    pub fn relative_to(parent: &impl IsA<gtk4::Widget>, pos: PositionType) -> Self {
        let menu = Self::new(parent);
        menu.popover.set_has_arrow(true);
        menu.popover.set_position(pos);
        menu
    }

    /// Sets a custom minimum width for the context menu.
    pub fn with_width(self, width: i32) -> Self {
        self.vbox.set_width_request(width);
        self
    }

    /// Adds a CSS class to the popover.
    pub fn with_css_class(self, class: &str) -> Self {
        self.popover.add_css_class(class);
        self
    }

    /// Appends a standard clickable item.
    pub fn item(self, label: &str, icon: &str, on_click: impl Fn() + 'static) -> Self {
        self.append_item(create_menu_item(label, icon), on_click)
    }

    /// Appends a clickable item using a gio::Icon.
    pub fn item_with_gicon(
        self,
        label: &str,
        icon: &impl IsA<gtk4::gio::Icon>,
        on_click: impl Fn() + 'static,
    ) -> Self {
        self.append_item(create_menu_item_gicon(label, icon), on_click)
    }

    /// Appends a clickable item resolving icon name or file path.
    pub fn item_with_icon_name(
        self,
        label: &str,
        icon_name_or_path: &str,
        on_click: impl Fn() + 'static,
    ) -> Self {
        self.append_item(
            create_menu_item_resolved(label, icon_name_or_path),
            on_click,
        )
    }

    /// Appends a standard clickable item with sensitivity control.
    pub fn item_sensitive(
        self,
        label: &str,
        icon: &str,
        sensitive: bool,
        on_click: impl Fn() + 'static,
    ) -> Self {
        self.append_item(create_menu_sens(label, icon, sensitive), on_click)
    }

    /// Appends an item with a keyboard shortcut hint.
    pub fn item_with_shortcut(
        self,
        label: &str,
        icon: &str,
        shortcut: &str,
        on_click: impl Fn() + 'static,
    ) -> Self {
        self.append_item(create_menu_shortcut(label, icon, shortcut), on_click)
    }

    /// Appends a destructive/danger item (e.g. Delete).
    pub fn destructive_item(self, label: &str, icon: &str, on_click: impl Fn() + 'static) -> Self {
        self.append_item(create_danger_item(label, icon), on_click)
    }

    /// Appends a destructive/danger item with sensitivity control.
    pub fn danger_sensitive(
        self,
        label: &str,
        icon: &str,
        sensitive: bool,
        on_click: impl Fn() + 'static,
    ) -> Self {
        self.append_item(create_danger_btn(label, icon, sensitive), on_click)
    }

    /// Appends a horizontal separator.
    pub fn separator(self) -> Self {
        self.vbox.append(&create_menu_sep());
        self
    }

    /// Appends a section group header.
    pub fn group_header(self, label: &str) -> Self {
        self.vbox.append(&create_group_header(label));
        self
    }

    /// Appends a hierarchical submenu item with instant hover display.
    pub fn submenu(
        self,
        label: &str,
        icon: Option<&str>,
        build_sub: impl FnOnce(ContextMenu) -> ContextMenu,
    ) -> Self {
        let btn = create_submenu_item(label, icon, true);
        let sub_popover = create_submenu_popover(&btn, "explore-popover");

        let mut all_ancestors = self.ancestor_popovers.clone();
        all_ancestors.push(self.popover.clone());

        let sub_builder = ContextMenu {
            popover: sub_popover.clone(),
            vbox: create_menu_box(180),
            footer_box: None,
            active_subpopover: Rc::new(RefCell::new(None)),
            ancestor_popovers: all_ancestors,
        };

        let sub_built = build_sub(sub_builder);
        sub_popover.set_child(Some(&sub_built.vbox));

        let sub_pop_click = sub_popover.clone();
        btn.connect_clicked(move |_| {
            sub_pop_click.popup();
        });

        let sub_pop_hover = sub_popover.clone();
        let active_sub = self.active_subpopover.clone();
        let motion = gtk4::EventControllerMotion::new();
        motion.connect_enter(move |_, _, _| {
            let mut cur = active_sub.borrow_mut();
            if let Some(prev) = cur.take() {
                if prev != sub_pop_hover {
                    prev.popdown();
                }
            }
            sub_pop_hover.popup();
            *cur = Some(sub_pop_hover.clone());
        });
        btn.add_controller(motion);

        self.vbox.append(&btn);
        self
    }

    /// Allows custom items to be appended directly within the builder chain.
    pub fn custom_items(self, build_fn: impl FnOnce(&Box, &Popover)) -> Self {
        build_fn(&self.vbox, &self.popover);
        self
    }

    /// Appends an arbitrary custom widget inside the context menu.
    pub fn custom_widget(self, widget: &impl IsA<gtk4::Widget>) -> Self {
        self.vbox.append(widget);
        self
    }

    /// Appends a raw button widget directly into the context menu.
    pub fn raw_item(self, button: &Button) -> Self {
        self.vbox.append(button);
        self
    }

    /// Appends a quick action icon button to the footer row.
    pub fn footer_button(
        mut self,
        icon: &str,
        tooltip: &str,
        on_click: impl Fn() + 'static,
    ) -> Self {
        let footer_box = self.ensure_footer_box();
        let btn = create_footer_btn(icon, tooltip);
        self.wire_click(&btn, on_click);
        let active_sub = self.active_subpopover.clone();
        let motion = gtk4::EventControllerMotion::new();
        motion.connect_enter(move |_, _, _| {
            if let Some(pop) = active_sub.borrow_mut().take() {
                pop.popdown();
            }
        });
        btn.add_controller(motion);
        footer_box.append(&btn);
        self
    }

    /// Appends a quick action icon button to the footer row with sensitivity control.
    pub fn footer_sensitive(
        mut self,
        icon: &str,
        tooltip: &str,
        sensitive: bool,
        on_click: impl Fn() + 'static,
    ) -> Self {
        let footer_box = self.ensure_footer_box();
        let btn = create_footer_btn(icon, tooltip);
        btn.set_sensitive(sensitive);
        self.wire_click(&btn, on_click);
        let active_sub = self.active_subpopover.clone();
        let motion = gtk4::EventControllerMotion::new();
        motion.connect_enter(move |_, _, _| {
            if let Some(pop) = active_sub.borrow_mut().take() {
                pop.popdown();
            }
        });
        btn.add_controller(motion);
        footer_box.append(&btn);
        self
    }

    /// Appends an arbitrary widget into the menu container.
    pub fn append<W: IsA<gtk4::Widget>>(&self, widget: &W) {
        self.vbox.append(widget);
    }

    /// Returns a reference to the underlying Popover.
    pub fn widget(&self) -> &Popover {
        &self.popover
    }

    /// Returns a reference to the internal vertical Box container.
    pub fn container(&self) -> &Box {
        &self.vbox
    }

    /// Displays the context menu immediately.
    pub fn popup(&self) {
        self.popover.popup();
    }

    /// Dismisses the context menu.
    pub fn popdown(&self) {
        self.popover.popdown();
    }

    /// Builds and returns `(Popover, Box)` container without opening.
    pub fn build(self) -> (Popover, Box) {
        (self.popover, self.vbox)
    }

    /// Wires a button click to dismiss the popover (and all ancestor popovers) and then run `on_click`.
    fn wire_click(&self, btn: &Button, on_click: impl Fn() + 'static) {
        let pop = self.popover.clone();
        let ancestors = self.ancestor_popovers.clone();
        let callback = Rc::new(on_click);
        btn.connect_clicked(move |_| {
            pop.popdown();
            for ancestor in &ancestors {
                ancestor.popdown();
            }
            callback();
        });
    }

    /// Appends a wired clickable button to the menu container.
    fn append_item(self, btn: Button, on_click: impl Fn() + 'static) -> Self {
        self.wire_click(&btn, on_click);
        let active_sub = self.active_subpopover.clone();
        let motion = gtk4::EventControllerMotion::new();
        motion.connect_enter(move |_, _, _| {
            if let Some(pop) = active_sub.borrow_mut().take() {
                pop.popdown();
            }
        });
        btn.add_controller(motion);
        self.vbox.append(&btn);
        self
    }

    /// Returns the footer button box, creating and attaching it on first use.
    fn ensure_footer_box(&mut self) -> Box {
        if let Some(fb) = &self.footer_box {
            return fb.clone();
        }

        let (footer_container, fb) = create_footer_box();
        self.vbox.append(&footer_container);
        self.footer_box = Some(fb.clone());
        fb
    }
}

