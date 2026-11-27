//! widget helpers.

use super::models::{notify_visibility, wire_row_hover};
use super::*;
use crate::components::overlay::popup_fit;

impl ContextMenu {
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
    /// Displays the context menu immediately, keeping it inside the monitor.
    pub fn popup(&self) {
        popup_fit(&self.popover, self.popover.position());
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
    pub(super) fn wire_click(&self, btn: &Button, on_click: impl Fn() + 'static) {
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
    pub(super) fn append_item(self, btn: Button, on_click: impl Fn() + 'static) -> Self {
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
    pub(super) fn ensure_footer_box(&mut self) -> Box {
        if let Some(fb) = &self.footer_box {
            return fb.clone();
        }

        let (footer_container, fb) = create_footer_box();
        self.vbox.append(&footer_container);
        self.footer_box = Some(fb.clone());
        fb
    }
}
