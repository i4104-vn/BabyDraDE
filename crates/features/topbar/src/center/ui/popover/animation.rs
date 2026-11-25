//! Toggle and cancelable slide-out animation helpers.

use std::cell::Cell;
use std::rc::Rc;

use animation_gtk::{slide_out_cb_cancelable, SlideDirection};
use gtk4::prelude::*;

pub(super) fn toggle(
    popover: &gtk4::Popover,
    content: &gtk4::Box,
    animating: &Rc<Cell<bool>>,
    generation: &Rc<Cell<u64>>,
    duration_ms: u64,
) {
    if animating.get() || popover.parent().is_none() || popover.root().is_none() {
        return;
    }
    if popover.is_visible() {
        popdown_animated(popover, content, animating, generation, duration_ms, || {});
    } else {
        popover.popup();
    }
}

fn popdown_animated<F: FnOnce() + 'static>(
    popover: &gtk4::Popover,
    content: &gtk4::Box,
    animating: &Rc<Cell<bool>>,
    generation: &Rc<Cell<u64>>,
    duration_ms: u64,
    on_finish: F,
) {
    if animating.get() {
        generation.set(generation.get().wrapping_add(1));
        animating.set(false);
    }
    if popover.parent().is_none() || popover.root().is_none() || !popover.is_visible() {
        if popover.parent().is_some() && popover.root().is_some() {
            popover.popdown();
        }
        on_finish();
        return;
    }
    animating.set(true);
    let current = generation.get().wrapping_add(1);
    generation.set(current);

    let popover = popover.clone();
    let content = content.clone();
    let animating = animating.clone();
    let generation = generation.clone();
    slide_out_cb_cancelable(
        content.upcast_ref(),
        SlideDirection::Up,
        15,
        duration_ms,
        false,
        generation.clone(),
        current,
        move || {
            if generation.get() != current {
                return;
            }
            if popover.parent().is_some() && popover.root().is_some() {
                popover.popdown();
            }
            animating.set(false);
            on_finish();
        },
    );
}
