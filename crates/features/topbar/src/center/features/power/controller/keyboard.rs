//! Keyboard navigation for the power popover (uses the shared context).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::gdk::Key;
use gtk4::glib;

use crate::center::features::power::ui::{execute_power_action, highlight_selection, PowerPopover};
use crate::center::IslandCtx;

pub fn create_keyboard_controller(
    selected: Rc<Cell<usize>>,
    popover_rc: Rc<RefCell<Option<PowerPopover>>>,
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

        if !popover_rc.borrow().as_ref().is_some_and(|p| p.is_visible()) {
            return glib::Propagation::Proceed;
        }

        let execute = |index: usize| {
            if let Some(popover) = popover_rc.borrow().as_ref() {
                execute_power_action(index, popover);
                ctx.release_override();
                ctx.hide();
            }
        };

        match keyval {
            Key::Left | Key::Up | Key::h | Key::k | Key::ISO_Left_Tab => {
                let previous = if selected.get() == 0 { 3 } else { selected.get() - 1 };
                selected.set(previous);
                if let Some(popover) = popover_rc.borrow().as_ref() {
                    highlight_selection(popover, previous);
                }
                glib::Propagation::Stop
            }
            Key::Right | Key::Down | Key::l | Key::j | Key::Tab => {
                let next = (selected.get() + 1) % 4;
                selected.set(next);
                if let Some(popover) = popover_rc.borrow().as_ref() {
                    highlight_selection(popover, next);
                }
                glib::Propagation::Stop
            }
            Key::Return | Key::KP_Enter | Key::space => {
                execute(selected.get());
                glib::Propagation::Stop
            }
            Key::_1 | Key::KP_1 => {
                selected.set(0);
                execute(0);
                glib::Propagation::Stop
            }
            Key::_2 | Key::KP_2 => {
                selected.set(1);
                execute(1);
                glib::Propagation::Stop
            }
            Key::_3 | Key::KP_3 => {
                selected.set(2);
                execute(2);
                glib::Propagation::Stop
            }
            Key::_4 | Key::KP_4 => {
                selected.set(3);
                execute(3);
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        }
    });

    controller
}
