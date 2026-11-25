//! Map/unmap lifecycle: CSS state, focus and slide-in animation.

use std::time::Duration;

use animation_gtk::slide_in;
use gtk4::prelude::*;
use gtk4_layer_shell::{KeyboardMode, LayerShell};

/// Modal island popovers briefly switch the panel's layer keyboard mode.
fn set_layer_keyboard_mode(widget: &impl IsA<gtk4::Widget>, mode: KeyboardMode) {
    if let Some(root) = widget.root() {
        if let Some(window) = root.downcast_ref::<gtk4::Window>() {
            if window.is_layer_window() {
                window.set_keyboard_mode(mode);
            }
        }
    }
}

/// True when another popover is still open on the capsule.
fn other_popover_open(capsule: &gtk4::Box, current: &gtk4::Popover) -> bool {
    let mut next = capsule.first_child();
    while let Some(child) = next {
        next = child.next_sibling();
        if let Some(popover) = child.downcast_ref::<gtk4::Popover>() {
            if popover.is_visible()
                && popover.upcast_ref::<gtk4::Widget>() != current.upcast_ref::<gtk4::Widget>()
            {
                return true;
            }
        }
    }
    false
}

/// Slide lifecycle used by media / notification popovers.
pub(super) fn slide(popover: &gtk4::Popover, capsule: &gtk4::Box, content: &gtk4::Box, ms: u64) {
    let box_slide = content.clone();
    let capsule_map = capsule.clone();
    popover.connect_map(move |_| {
        capsule_map.add_css_class("popover-open");
        slide_in(box_slide.upcast_ref(), animation_gtk::SlideDirection::Down, 15, ms);
    });

    let capsule_unmap = capsule.clone();
    popover.connect_unmap(move |popover| {
        if !other_popover_open(&capsule_unmap, popover) {
            capsule_unmap.remove_css_class("popover-open");
        }
    });
}

/// Modal lifecycle used by power / clipboard / recording popovers.
pub(super) fn modal(popover: &gtk4::Popover, capsule: &gtk4::Box, content: &gtk4::Box, ms: u64) {
    let box_focus = content.clone();
    let box_slide = content.clone();
    let capsule_map = capsule.clone();
    let pop_focus = popover.clone();
    popover.connect_map(move |_| {
        capsule_map.add_css_class("popover-open");
        set_layer_keyboard_mode(&pop_focus, KeyboardMode::OnDemand);
        let focus_box = box_focus.clone();
        let focus_popover = pop_focus.clone();
        gtk4::glib::timeout_add_local_once(Duration::from_millis(40), move || {
            focus_popover.grab_focus();
            focus_box.grab_focus();
        });
        slide_in(box_slide.upcast_ref(), animation_gtk::SlideDirection::Down, 15, ms);
    });

    let capsule_unmap = capsule.clone();
    popover.connect_unmap(move |popover| {
        if !other_popover_open(&capsule_unmap, popover) {
            capsule_unmap.remove_css_class("popover-open");
        }
        set_layer_keyboard_mode(popover, KeyboardMode::OnDemand);
    });
}
