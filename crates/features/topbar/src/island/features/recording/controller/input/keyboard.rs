//! Keyboard navigation for the recording popover (uses the shared context).

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::gdk::Key;
use gtk4::glib;

use crate::island::features::recording::service;
use crate::island::features::recording::ui::RecordingPopover;
use crate::island::IslandCtx;

pub fn create_keyboard_controller(
    popover_rc: Rc<RefCell<Option<RecordingPopover>>>,
    ctx: IslandCtx,
) -> gtk4::EventControllerKey {
    let controller = gtk4::EventControllerKey::new();

    controller.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == Key::Escape {
            if let Some(popover) = popover_rc.borrow().as_ref() {
                popover.popdown();
            }
            if !service::is_recording() {
                ctx.release_override();
                ctx.hide();
            }
            ctx.dismiss_popovers();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });

    controller
}
