//! Keyboard navigation for the clipboard popover (uses the shared context).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::gdk::Key;
use gtk4::glib;

use crate::island::features::clipboard::service;
use crate::island::features::clipboard::ui::{render_popover, ClipboardPopover};
use crate::island::IslandCtx;

/// Creates an event controller key that navigates and selects clipboard entries.
pub fn create_keyboard_controller(
    selected: Rc<Cell<usize>>,
    popover_rc: Rc<RefCell<Option<ClipboardPopover>>>,
    ctx: IslandCtx,
) -> gtk4::EventControllerKey {
    let controller = gtk4::EventControllerKey::new();

    controller.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == Key::Escape {
            if let Some(popover) = popover_rc.borrow().as_ref() {
                popover.popdown();
            }
            ctx.release_override();
            ctx.hide();
            ctx.dismiss_popovers();
            return glib::Propagation::Stop;
        }

        let is_open = popover_rc.borrow().as_ref().is_some_and(|p| p.is_visible());
        if !is_open {
            return glib::Propagation::Proceed;
        }

        let entries = service::get_entries();
        let copy_and_dismiss = |index: usize| {
            if let Some(entry) = entries.get(index) {
                service::copy_to_system_clipboard(entry);
            }
            if let Some(popover) = popover_rc.borrow().as_ref() {
                popover.popdown();
            }
            ctx.release_override();
            ctx.hide();
        };

        match keyval {
            Key::Up | Key::k | Key::ISO_Left_Tab => {
                if !entries.is_empty() {
                    let next = if selected.get() == 0 {
                        entries.len() - 1
                    } else {
                        selected.get() - 1
                    };
                    selected.set(next);
                    if let Some(popover) = popover_rc.borrow().as_ref() {
                        render_popover(popover, &entries, next);
                    }
                }
                glib::Propagation::Stop
            }
            Key::Down | Key::j | Key::Tab => {
                if !entries.is_empty() {
                    let next = (selected.get() + 1) % entries.len();
                    selected.set(next);
                    if let Some(popover) = popover_rc.borrow().as_ref() {
                        render_popover(popover, &entries, next);
                    }
                }
                glib::Propagation::Stop
            }
            Key::Return | Key::KP_Enter | Key::space => {
                if entries.is_empty() {
                    if let Some(popover) = popover_rc.borrow().as_ref() {
                        popover.popdown();
                    }
                } else {
                    copy_and_dismiss(selected.get());
                }
                glib::Propagation::Stop
            }
            Key::_1 | Key::KP_1 if !entries.is_empty() => {
                selected.set(0);
                copy_and_dismiss(0);
                glib::Propagation::Stop
            }
            Key::_2 | Key::KP_2 if entries.len() > 1 => {
                selected.set(1);
                copy_and_dismiss(1);
                glib::Propagation::Stop
            }
            Key::_3 | Key::KP_3 if entries.len() > 2 => {
                selected.set(2);
                copy_and_dismiss(2);
                glib::Propagation::Stop
            }
            Key::_4 | Key::KP_4 if entries.len() > 3 => {
                selected.set(3);
                copy_and_dismiss(3);
                glib::Propagation::Stop
            }
            Key::_5 | Key::KP_5 if entries.len() > 4 => {
                selected.set(4);
                copy_and_dismiss(4);
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        }
    });

    controller
}
