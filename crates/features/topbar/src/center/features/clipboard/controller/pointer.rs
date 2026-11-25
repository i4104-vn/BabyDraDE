//! Pointer interactions for the clipboard popover (rows + wheel).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;

use crate::center::features::clipboard::service;
use crate::center::features::clipboard::ui::{render_popover, ClipboardPopover, MAX_VISIBLE_ITEMS};
use crate::center::IslandCtx;

/// Wires row clicks and mouse-wheel navigation through the shared context.
pub(crate) fn wire(
    popover: &ClipboardPopover,
    selected: Rc<Cell<usize>>,
    popover_rc: Rc<RefCell<Option<ClipboardPopover>>>,
    ctx: &IslandCtx,
) {
    for (row_index, row) in popover.rows.iter().enumerate() {
        let selected = selected.clone();
        let popover_widget = popover.popover.clone();
        let row_ctx = ctx.clone();
        row.click_gesture.connect_pressed(move |_, _, _, _| {
            let entries = service::get_entries();
            if entries.is_empty() {
                return;
            }
            let start = if entries.len() <= MAX_VISIBLE_ITEMS {
                0
            } else if selected.get() >= 3 {
                (selected.get() - 2).min(entries.len() - MAX_VISIBLE_ITEMS)
            } else {
                0
            };
            if let Some(entry) = entries.get(start + row_index) {
                service::copy_to_system_clipboard(entry);
            }
            popover_widget.popdown();
            row_ctx.release_override();
            row_ctx.hide();
            row_ctx.tick();
        });
    }

    for target in [
        popover.popover.clone().upcast::<gtk4::Widget>(),
        popover.popover_box.clone().upcast::<gtk4::Widget>(),
    ] {
        let selected = selected.clone();
        let popover_rc = popover_rc.clone();
        let scroll = gtk4::EventControllerScroll::new(
            gtk4::EventControllerScrollFlags::VERTICAL | gtk4::EventControllerScrollFlags::DISCRETE,
        );
        scroll.connect_scroll(move |_, _, dy| {
            let entries = service::get_entries();
            if !entries.is_empty() {
                let current = selected.get();
                let next = if dy > 0.0 {
                    (current + 1) % entries.len()
                } else if dy < 0.0 {
                    if current == 0 {
                        entries.len() - 1
                    } else {
                        current - 1
                    }
                } else {
                    current
                };
                selected.set(next);
                if let Some(popover) = popover_rc.borrow().as_ref() {
                    render_popover(popover, &entries, next);
                }
            }
            gtk4::glib::Propagation::Stop
        });
        target.add_controller(scroll);
    }
}
