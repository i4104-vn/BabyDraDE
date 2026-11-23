//! Fluent builder for declaratively constructing and presenting context menus.

use super::items::*;
use gtk4::prelude::*;
use gtk4::{Box, Button, Popover, PositionType};
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    /// Every top-level context menu is registered here so shells can dismiss
    /// all of them on a single outside click (layer-shell panels never see
    /// clicks that land on other surfaces).
    static OPEN_MENUS: RefCell<Vec<Popover>> = const { RefCell::new(Vec::new()) };
    /// Shells register a hook here to show/hide their outside-click catcher.
    static VISIBILITY_HOOK: RefCell<Option<std::boxed::Box<dyn Fn(bool)>>> =
        const { RefCell::new(None) };
}

/// Lets the shell show/hide a full-screen click catcher whenever any context
/// menu opens or closes (layer-shell panels cannot see clicks on other surfaces).
pub fn set_visibility_hook(hook: impl Fn(bool) + 'static) {
    VISIBILITY_HOOK.with(|slot| *slot.borrow_mut() = Some(std::boxed::Box::new(hook)));
}

fn notify_visibility(open: bool) {
    VISIBILITY_HOOK.with(|slot| {
        if let Some(hook) = slot.borrow().as_ref() {
            hook(open);
        }
    });
}

/// Closes every open context menu across the application.
pub fn dismiss_all() {
    let popovers = OPEN_MENUS.with(|menus| menus.borrow().clone());
    for popover in popovers {
        if popover.is_visible() {
            popover.popdown();
        }
    }
}

/// Adds the shared hover highlight to a menu row. The CSS class works even
/// when a system theme overrides the `:hover` pseudo-class.
fn wire_row_hover(btn: &Button) {
    let motion = gtk4::EventControllerMotion::new();
    let enter = btn.downgrade();
    let leave = btn.downgrade();
    motion.connect_enter(move |_, _, _| {
        if let Some(btn) = enter.upgrade() {
            btn.add_css_class("context-menu-row-hover");
        }
    });
    motion.connect_leave(move |_| {
        if let Some(btn) = leave.upgrade() {
            btn.remove_css_class("context-menu-row-hover");
        }
    });
    btn.add_controller(motion);
}

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
        let closed_popover = popover.clone();
        popover.connect_closed(move |_| {
            if let Some(sub) = active_sub_close.borrow_mut().take() {
                sub.popdown();
            }
            OPEN_MENUS.with(|menus| {
                menus.borrow_mut().retain(|menu| menu != &closed_popover);
            });
            notify_visibility(false);
        });

        let escape = gtk4::EventControllerKey::new();
        let escape_popover = popover.clone();
        escape.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape && escape_popover.is_visible() {
                escape_popover.popdown();
                return gtk4::glib::Propagation::Stop;
            }
            gtk4::glib::Propagation::Proceed
        });
        popover.add_controller(escape);

        OPEN_MENUS.with(|menus| menus.borrow_mut().push(popover.clone()));

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

    /// Sets the side on which this context menu opens relative to its parent.
    pub fn with_position(self, position: PositionType) -> Self {
        self.popover.set_position(position);
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
        wire_row_hover(&btn);

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
        notify_visibility(true);
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
        wire_row_hover(&btn);
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
