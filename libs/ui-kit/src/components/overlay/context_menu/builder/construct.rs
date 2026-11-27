//! construct helpers.

use super::models::{notify_visibility, OPEN_MENUS};
use super::*;
use crate::components::overlay::{max_popover_height, max_popover_width, EDGE_MARGIN};

impl ContextMenu {
    /// Starts building a context menu attached to `parent`.
    pub fn new(parent: &impl IsA<gtk4::Widget>) -> Self {
        let popover = Popover::builder().has_arrow(false).autohide(true).build();
        popover.set_parent(parent.as_ref());
        popover.add_css_class("context-menu-popover");
        popover.add_css_class("explore-popover");
        popover.add_css_class("desktop-context-menu");

        let vbox = create_menu_box(200);
        // Long menus must not run past the monitor bottom. Keep the original
        // `vbox` API for callers, but host it in a capped scrolled window.
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_hscrollbar_policy(gtk4::PolicyType::Never);
        scrolled.set_vscrollbar_policy(gtk4::PolicyType::Automatic);
        scrolled.set_propagate_natural_width(true);
        scrolled.set_propagate_natural_height(true);
        if let Some(max_height) = max_popover_height(parent.as_ref(), EDGE_MARGIN) {
            scrolled.set_max_content_height(max_height);
        }
        if let Some(max_width) = max_popover_width(parent.as_ref(), EDGE_MARGIN) {
            scrolled.set_max_content_width(max_width);
        }
        scrolled.set_child(Some(&vbox));
        popover.set_child(Some(&scrolled));

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
}
