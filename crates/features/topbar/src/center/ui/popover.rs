//! Shared slide animation and lifecycle for Dynamic Island popovers.
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;

use animation::{slide_in, slide_out_cb_cancelable, SlideDirection};

thread_local! {
    static OPEN_POPOVERS: RefCell<Vec<gtk4::Popover>> = const { RefCell::new(Vec::new()) };
    static ISLAND_CAPSULE: RefCell<Option<gtk4::Box>> = const { RefCell::new(None) };
}

/// Dismisses every island popover (used by outside-click / Escape handlers).
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

#[derive(Clone)]
pub struct IslandPopover {
    pub popover: gtk4::Popover,
    pub popover_box: gtk4::Box,
    pub is_animating: Rc<Cell<bool>>,
    animation_generation: Rc<Cell<u64>>,
    duration_ms: u64,
}

impl IslandPopover {
    /// Creates a slide-animated popover anchored to the island capsule.
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

        let popover_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        popover_box.add_css_class(box_css_class);
        popover.set_focusable(false);
        popover.set_can_focus(false);
        popover_box.set_focusable(false);
        popover_box.set_can_focus(false);
        popover.set_child(Some(&popover_box));

        let escape = gtk4::EventControllerKey::new();
        let pop_esc = popover.clone();
        escape.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape && pop_esc.is_visible() {
                pop_esc.popdown();
                return gtk4::glib::Propagation::Stop;
            }
            gtk4::glib::Propagation::Proceed
        });
        popover.add_controller(escape);

        OPEN_POPOVERS.with(|list| list.borrow_mut().push(popover.clone()));
        ISLAND_CAPSULE.with(|slot| *slot.borrow_mut() = Some(capsule.clone()));

        setup_popover_slide_lifecycle(&popover, capsule, &popover_box, duration_ms);

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
        toggle_popover_animated(
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

    pub fn is_animating(&self) -> bool {
        self.is_animating.get()
    }
}

pub fn toggle_popover_animated(
    popover: &gtk4::Popover,
    content_box: &gtk4::Box,
    is_animating: &Rc<Cell<bool>>,
    animation_generation: &Rc<Cell<u64>>,
    duration_ms: u64,
) {
    if is_animating.get() || popover.parent().is_none() || popover.root().is_none() {
        return;
    }
    if popover.is_visible() {
        popdown_animated_cb(
            popover,
            content_box,
            is_animating,
            animation_generation,
            duration_ms,
            || {},
        );
    } else {
        popover.popup();
    }
}

pub fn popdown_animated_cb<F: FnOnce() + 'static>(
    popover: &gtk4::Popover,
    content_box: &gtk4::Box,
    is_animating: &Rc<Cell<bool>>,
    animation_generation: &Rc<Cell<u64>>,
    duration_ms: u64,
    on_finish: F,
) {
    if is_animating.get() {
        animation_generation.set(animation_generation.get().wrapping_add(1));
        is_animating.set(false);
    }
    if popover.parent().is_none() || popover.root().is_none() || !popover.is_visible() {
        if popover.parent().is_some() && popover.root().is_some() {
            popover.popdown();
        }
        on_finish();
        return;
    }
    is_animating.set(true);
    let generation = animation_generation.get().wrapping_add(1);
    animation_generation.set(generation);
    let pop_c = popover.clone();
    let box_c = content_box.clone();
    let anim_c = is_animating.clone();
    let generation_c = animation_generation.clone();
    slide_out_cb_cancelable(
        box_c.upcast_ref(),
        SlideDirection::Up,
        15,
        duration_ms,
        false,
        generation_c.clone(),
        generation,
        move || {
            if generation_c.get() != generation {
                return;
            }
            if pop_c.parent().is_some() && pop_c.root().is_some() {
                pop_c.popdown();
            }
            anim_c.set(false);
            on_finish();
        },
    );
}

pub fn setup_popover_slide_lifecycle(
    popover: &gtk4::Popover,
    capsule: &gtk4::Box,
    content_box: &gtk4::Box,
    duration_ms: u64,
) {
    let box_slide = content_box.clone();
    let capsule_map = capsule.clone();
    popover.connect_map(move |_| {
        capsule_map.add_css_class("popover-open");
        slide_in(
            box_slide.upcast_ref(),
            SlideDirection::Down,
            15,
            duration_ms,
        );
    });

    let capsule_unmap = capsule.clone();
    popover.connect_unmap(move |p| {
        let mut other_open = false;
        let mut next = capsule_unmap.first_child();
        while let Some(child) = next {
            next = child.next_sibling();
            if let Some(pop) = child.downcast_ref::<gtk4::Popover>() {
                if pop.is_visible()
                    && pop.upcast_ref::<gtk4::Widget>() != p.upcast_ref::<gtk4::Widget>()
                {
                    other_open = true;
                    break;
                }
            }
        }
        if !other_open {
            capsule_unmap.remove_css_class("popover-open");
        }
    });
}
