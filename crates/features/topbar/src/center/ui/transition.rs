//! Dynamic Island transition animations and view state switching.
//! Adapted verbatim from the old island `ui/transition.rs`.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;

use crate::center::engine::IslandEngine;
use animation::{current_anim_gen, island_animate_size, island_zoom_in, island_zoom_out};

const EXPAND_MS: u64 = 350;
const COLLAPSE_MS: u64 = 500;

/// Applies a transition to the new display state, animating the capsule.
pub(crate) fn apply_transition(
    engine: &mut IslandEngine,
    desired: Option<usize>,
    engine_rc: &Rc<RefCell<IslandEngine>>,
) {
    // Dismiss popovers when the island is collapsing to hidden.
    if desired.is_none() {
        let mut next = engine.capsule.first_child();
        while let Some(child) = next {
            next = child.next_sibling();
            if let Some(popover) = child.downcast_ref::<gtk4::Popover>() {
                if popover.is_visible() {
                    popover.popdown();
                }
            }
        }
    }

    // Hide the previously displayed view.
    if let Some(prev) = engine.displayed {
        let view = &engine.views[prev];
        view.state.active.set(false);
        view.container.set_visible(false);
        if let Some(cls) = view.feature.borrow().capsule_class() {
            engine.capsule.remove_css_class(cls);
        }
        if let Ok(mut feature) = view.feature.try_borrow_mut() {
            feature.on_hide();
        }
    }
    engine.displayed = desired;

    match desired {
        Some(w) => {
            let view = &engine.views[w];
            view.container.set_visible(true);
            view.container.set_opacity(1.0);
            engine.capsule.set_visible(true);
            engine.capsule.set_opacity(1.0);
            if let Some(cls) = view.feature.borrow().capsule_class() {
                engine.capsule.add_css_class(cls);
            }
            if let Ok(mut feature) = view.feature.try_borrow_mut() {
                feature.on_show();
            }
            let size = view.feature.borrow().size();
            animate_expand(engine, size, true, engine_rc);
        }
        None => animate_collapse(engine, engine_rc),
    }
}

/// Expands the capsule to `target` with a zoom/size animation.
fn animate_expand(
    engine: &mut IslandEngine,
    target: (i32, i32),
    active_music: bool,
    engine_rc: &Rc<RefCell<IslandEngine>>,
) {
    engine.animating.set(true);
    let capsule = engine.capsule.clone();
    let (tw, th) = target;
    let cur_w = capsule.width().max(0);
    let cur_h = capsule.height().max(0);
    let ms = EXPAND_MS;
    let rc2 = engine_rc.clone();

    capsule.set_visible(true);
    capsule.set_opacity(1.0);
    let mut next = capsule.first_child();
    while let Some(child) = next {
        next = child.next_sibling();
        child.set_opacity(1.0);
    }
    if active_music {
        capsule.add_css_class("active-music");
    }

    if cur_w <= 0 || cur_h <= 0 {
        island_zoom_in(capsule.upcast_ref(), tw, th, ms);
        let anim_gen = current_anim_gen();
        glib::timeout_add_local_once(Duration::from_millis(ms + 60), move || {
            if current_anim_gen() != anim_gen {
                return;
            }
            if let Ok(core) = rc2.try_borrow_mut() {
                core.animating.set(false);
            }
        });
    } else {
        let rc_on_complete = engine_rc.clone();
        island_animate_size(capsule.upcast_ref(), cur_w, tw, cur_h, th, ms, move || {
            if let Ok(core) = rc_on_complete.try_borrow_mut() {
                core.animating.set(false);
            }
        });
        let anim_gen = current_anim_gen();
        let rc3 = engine_rc.clone();
        glib::timeout_add_local_once(Duration::from_millis(ms + 120), move || {
            if current_anim_gen() != anim_gen {
                return;
            }
            if let Ok(core) = rc3.try_borrow_mut() {
                core.animating.set(false);
            }
        });
    }
}

/// Collapses the capsule back to hidden.
fn animate_collapse(engine: &mut IslandEngine, engine_rc: &Rc<RefCell<IslandEngine>>) {
    engine.animating.set(true);
    let capsule = engine.capsule.clone();
    let cur_w = capsule.width().max(1);
    let ms = COLLAPSE_MS;
    let rc2 = engine_rc.clone();

    island_zoom_out(capsule.upcast_ref(), cur_w, ms, true);
    let anim_gen = current_anim_gen();
    glib::timeout_add_local_once(Duration::from_millis(ms + 60), move || {
        if current_anim_gen() != anim_gen {
            return;
        }
        if let Ok(core) = rc2.try_borrow_mut() {
            // Only hide if the island state is STILL hidden.
            if core.displayed.is_none() {
                core.animating.set(false);
                core.capsule.remove_css_class("active-music");
                core.capsule.remove_css_class("notification-mode");
                core.capsule.set_visible(false);
            }
        }
    });
}
