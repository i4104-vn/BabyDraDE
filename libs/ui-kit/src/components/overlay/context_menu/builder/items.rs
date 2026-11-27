//! items helpers.

use super::models::wire_row_hover;
use super::*;
use crate::components::overlay::{max_popover_height, max_popover_width, popup_fit, EDGE_MARGIN};

impl ContextMenu {
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
        let sub_scrolled = gtk4::ScrolledWindow::new();
        sub_scrolled.set_hscrollbar_policy(gtk4::PolicyType::Never);
        sub_scrolled.set_vscrollbar_policy(gtk4::PolicyType::Automatic);
        sub_scrolled.set_propagate_natural_width(true);
        sub_scrolled.set_propagate_natural_height(true);
        let anchor = btn.upcast_ref::<gtk4::Widget>();
        if let Some(max_height) = max_popover_height(anchor, EDGE_MARGIN) {
            sub_scrolled.set_max_content_height(max_height);
        }
        if let Some(max_width) = max_popover_width(anchor, EDGE_MARGIN) {
            sub_scrolled.set_max_content_width(max_width);
        }
        sub_scrolled.set_child(Some(&sub_built.vbox));
        sub_popover.set_child(Some(&sub_scrolled));

        let sub_pop_click = sub_popover.clone();
        btn.connect_clicked(move |_| {
            popup_fit(&sub_pop_click, sub_pop_click.position());
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
            popup_fit(&sub_pop_hover, sub_pop_hover.position());
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
}
