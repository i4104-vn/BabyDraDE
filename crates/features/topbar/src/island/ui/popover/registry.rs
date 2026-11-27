//! Process-wide popover registry and outside-click dismissal.

use std::cell::RefCell;

use gtk4::prelude::*;

thread_local! {
    static OPEN_POPOVERS: RefCell<Vec<gtk4::Popover>> = const { RefCell::new(Vec::new()) };
    static ISLAND_CAPSULE: RefCell<Option<gtk4::Box>> = const { RefCell::new(None) };
}

/// Registers a new popover so it can be dismissed centrally.
pub(super) fn track(popover: &gtk4::Popover, capsule: &gtk4::Box) {
    OPEN_POPOVERS.with(|list| list.borrow_mut().push(popover.clone()));
    ISLAND_CAPSULE.with(|slot| *slot.borrow_mut() = Some(capsule.clone()));
}

/// Builds the shared Escape controller for a popover.
pub(super) fn escape_controller(popover: &gtk4::Popover) -> gtk4::EventControllerKey {
    let escape = gtk4::EventControllerKey::new();
    let popover = popover.clone();
    escape.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Escape && popover.is_visible() {
            popover.popdown();
            return gtk4::glib::Propagation::Stop;
        }
        gtk4::glib::Propagation::Proceed
    });
    escape
}

/// Dismisses every island popover (outside-click / Escape handlers).
pub fn dismiss_all_popovers() {
    let popovers = OPEN_POPOVERS.with(|list| list.borrow().clone());
    for popover in popovers {
        if popover.is_visible() {
            popover.popdown();
        }
    }
}

/// Dismisses island popovers when a click lands outside the capsule.
pub fn dismiss_outside(window: &gtk4::Widget, x: f64, y: f64) {
    let inside = ISLAND_CAPSULE.with(|slot| {
        slot.borrow().as_ref().is_some_and(|capsule| {
            capsule
                .translate_coordinates(window, 0.0, 0.0)
                .map(|(cx, cy)| {
                    x >= cx
                        && x <= cx + capsule.width() as f64
                        && y >= cy
                        && y <= cy + capsule.height() as f64
                })
                .unwrap_or(false)
        })
    });
    if !inside {
        dismiss_all_popovers();
    }
}
