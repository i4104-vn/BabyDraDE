//! Shared base for Dynamic Island popovers.
//!
//! - [`registry`]: process-wide popover tracking / dismissal
//! - [`lifecycle`]: map/unmap CSS state, focus and slide-in
//! - [`animation`]: toggle and cancelable slide-out

mod animation;
mod lifecycle;
mod registry;

pub use registry::{dismiss_all_popovers, dismiss_outside};

use std::cell::Cell;
use std::rc::Rc;

use gtk4::prelude::*;

#[derive(Clone)]
pub struct IslandPopover {
    pub popover: gtk4::Popover,
    pub popover_box: gtk4::Box,
    pub is_animating: Rc<Cell<bool>>,
    animation_generation: Rc<Cell<u64>>,
    duration_ms: u64,
}

impl IslandPopover {
    /// Creates a slide-animated popover (media player).
    pub fn new_slide(
        capsule: &gtk4::Box,
        css_class: &str,
        box_css_class: &str,
        duration_ms: u64,
    ) -> Self {
        let popover =
            ui_kit::components::create_popover(capsule, gtk4::PositionType::Bottom, css_class);
        popover.set_has_arrow(false);
        popover.set_offset(0, 10);
        popover.set_autohide(true);
        popover.set_focusable(false);
        popover.set_can_focus(false);

        let popover_box = Self::content_box(box_css_class, false);
        popover.set_child(Some(&popover_box));
        popover.add_controller(registry::escape_controller(&popover));
        registry::track(&popover, capsule);
        lifecycle::slide(&popover, capsule, &popover_box, duration_ms);

        Self::assemble(popover, popover_box, duration_ms)
    }

    /// Creates a focusable modal popover (power, clipboard, recording).
    pub fn new_modal(
        capsule: &gtk4::Box,
        css_class: &str,
        box_css_class: &str,
        duration_ms: u64,
    ) -> Self {
        let popover =
            ui_kit::components::create_popover(capsule, gtk4::PositionType::Bottom, css_class);
        popover.set_has_arrow(false);
        popover.set_offset(0, 10);
        popover.set_autohide(true);
        popover.set_focusable(true);

        let popover_box = Self::content_box(box_css_class, true);
        popover.set_child(Some(&popover_box));
        popover.add_controller(registry::escape_controller(&popover));
        registry::track(&popover, capsule);
        lifecycle::modal(&popover, capsule, &popover_box, duration_ms);

        Self::assemble(popover, popover_box, duration_ms)
    }

    fn content_box(css_class: &str, focusable: bool) -> gtk4::Box {
        let box_ = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        box_.add_css_class(css_class);
        box_.set_focusable(focusable);
        box_.set_can_focus(focusable);
        box_
    }

    fn assemble(popover: gtk4::Popover, popover_box: gtk4::Box, duration_ms: u64) -> Self {
        Self {
            popover,
            popover_box,
            is_animating: Rc::new(Cell::new(false)),
            animation_generation: Rc::new(Cell::new(0)),
            duration_ms,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.popover.is_visible() || self.popover.property::<bool>("visible")
    }

    pub fn root(&self) -> Option<gtk4::Root> {
        self.popover.root()
    }

    pub fn parent(&self) -> Option<gtk4::Widget> {
        self.popover.parent()
    }

    pub fn popup(&self) {
        if self.popover.parent().is_none() || self.popover.root().is_none() {
            return;
        }
        self.cancel_animation();
        self.popover.popup();
    }

    pub fn popdown(&self) {
        self.cancel_animation();
        if self.popover.parent().is_some() && self.popover.root().is_some() {
            self.popover.popdown();
        }
    }

    pub fn toggle(&self) {
        animation::toggle(
            &self.popover,
            &self.popover_box,
            &self.is_animating,
            &self.animation_generation,
            self.duration_ms,
        );
    }

    pub fn cancel_animation(&self) {
        self.animation_generation
            .set(self.animation_generation.get().wrapping_add(1));
        self.is_animating.set(false);
        self.popover_box.set_opacity(1.0);
        self.popover_box.set_margin_top(0);
        self.popover_box.set_margin_bottom(0);
    }
}
