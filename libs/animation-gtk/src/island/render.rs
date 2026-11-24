//! Widget-side rendering for Dynamic Island capsule transitions.

use std::cell::Cell;

use super::handler::{
    child_collapse_opacity, collapse_opacity, interpolate, reveal_opacity,
};
use animation::Easing;
use gtk4::prelude::*;

thread_local! {
    static CURRENT_ANIM_GEN: Cell<u64> = const { Cell::new(0) };
}

/// Returns the next generation token used to cancel stale animations.
///
/// # Returns
/// A monotonically increasing per-thread generation counter.
pub fn next_anim_gen() -> u64 {
    CURRENT_ANIM_GEN.with(|g| {
        let next = g.get() + 1;
        g.set(next);
        next
    })
}

/// Returns the most recently issued animation generation token.
///
/// # Returns
/// The current per-thread generation counter.
pub fn current_anim_gen() -> u64 {
    CURRENT_ANIM_GEN.with(|g| g.get())
}

/// Animates the Dynamic Island capsule expanding from a small pill to full width. Uses FrameClock timing.
pub fn island_zoom_in(
    widget: &gtk4::Widget,
    target_width: i32,
    target_height: i32,
    duration_ms: u64,
) {
    let anim_gen = next_anim_gen();
    widget.set_opacity(1.0);
    widget.set_visible(true);
    widget.set_size_request(target_height, target_height);

    if let Some(ref child) = widget.first_child() {
        child.set_opacity(0.0);
    }

    let start_time = std::cell::Cell::new(0i64);
    let dur_us = duration_ms as i64 * 1000;

    widget.add_tick_callback(move |w, clock| {
        // Cancel if a newer animation was started
        if current_anim_gen() != anim_gen {
            return glib::ControlFlow::Break;
        }

        let now = clock.frame_time();
        if start_time.get() == 0 {
            start_time.set(now);
        }
        let elapsed_us = now - start_time.get();
        if elapsed_us >= dur_us {
            w.set_size_request(target_width, target_height);
            w.set_opacity(1.0);
            if let Some(ref child) = w.first_child() {
                child.set_opacity(1.0);
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed_us as f64 / dur_us as f64;
        w.set_opacity(1.0);

        let eased_w = Easing::EaseOutBackSoft.apply(t);
        let current_w = interpolate(target_height, target_width, eased_w);
        w.set_size_request(current_w, target_height);

        if let Some(ref child) = w.first_child() {
            child.set_opacity(reveal_opacity(t));
        }

        glib::ControlFlow::Continue
    });
}

/// Animates the Dynamic Island capsule collapsing back to a pill. Uses FrameClock timing.
pub fn island_zoom_out(
    widget: &gtk4::Widget,
    target_width: i32,
    duration_ms: u64,
    hide_after: bool,
) {
    let anim_gen = next_anim_gen();
    let start_h = widget.height().max(22);

    if let Some(ref child) = widget.first_child() {
        child.set_opacity(1.0);
    }

    let start_time = std::cell::Cell::new(0i64);
    let dur_us = duration_ms as i64 * 1000;

    widget.add_tick_callback(move |w, clock| {
        // Cancel if a newer animation was started
        if current_anim_gen() != anim_gen {
            return glib::ControlFlow::Break;
        }

        let now = clock.frame_time();
        if start_time.get() == 0 {
            start_time.set(now);
        }
        let elapsed_us = now - start_time.get();
        if elapsed_us >= dur_us {
            w.set_size_request(0, 0);
            w.set_opacity(0.0);
            if let Some(ref child) = w.first_child() {
                child.set_opacity(0.0);
            }
            if hide_after {
                w.set_visible(false);
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed_us as f64 / dur_us as f64;
        w.set_opacity(collapse_opacity(t));

        let eased_w = Easing::EaseOutCubic.apply(t);
        let current_w = interpolate(start_h, target_width, 1.0 - eased_w);
        w.set_size_request(current_w, start_h);

        if let Some(ref child) = w.first_child() {
            child.set_opacity(child_collapse_opacity(t));
        }

        glib::ControlFlow::Continue
    });
}

/// Animates the width of the Dynamic Island capsule from start_width to target_width.
pub fn island_animate_width<F>(
    widget: &gtk4::Widget,
    start_width: i32,
    target_width: i32,
    duration_ms: u64,
    on_complete: F,
) where
    F: FnOnce() + 'static,
{
    let anim_gen = next_anim_gen();
    let start_time = std::cell::Cell::new(0i64);
    let dur_us = duration_ms as i64 * 1000;
    let on_complete_opt = std::cell::RefCell::new(Some(on_complete));

    widget.add_tick_callback(move |w, clock| {
        // Cancel if a newer animation was started
        if current_anim_gen() != anim_gen {
            return glib::ControlFlow::Break;
        }

        let now = clock.frame_time();
        if start_time.get() == 0 {
            start_time.set(now);
        }
        let elapsed_us = now - start_time.get();
        if elapsed_us >= dur_us {
            w.set_size_request(target_width, -1);
            if let Some(cb) = on_complete_opt.borrow_mut().take() {
                cb();
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed_us as f64 / dur_us as f64;
        let eased = Easing::EaseOutCubic.apply(t);
        w.set_size_request(interpolate(start_width, target_width, eased), -1);

        glib::ControlFlow::Continue
    });
}

/// Animates both the width and height of the Dynamic Island capsule.
pub fn island_animate_size<F>(
    widget: &gtk4::Widget,
    start_width: i32,
    target_width: i32,
    start_height: i32,
    target_height: i32,
    duration_ms: u64,
    on_complete: F,
) where
    F: FnOnce() + 'static,
{
    let anim_gen = next_anim_gen();
    widget.set_opacity(1.0);
    widget.set_visible(true);
    if let Some(ref child) = widget.first_child() {
        child.set_opacity(1.0);
    }

    let start_time = std::cell::Cell::new(0i64);
    let dur_us = duration_ms as i64 * 1000;
    let on_complete_opt = std::cell::RefCell::new(Some(on_complete));

    widget.add_tick_callback(move |w, clock| {
        // Cancel if a newer animation was started
        if current_anim_gen() != anim_gen {
            return glib::ControlFlow::Break;
        }

        let now = clock.frame_time();
        if start_time.get() == 0 {
            start_time.set(now);
        }
        let elapsed_us = now - start_time.get();
        if elapsed_us >= dur_us {
            w.set_size_request(target_width, target_height);
            let target_op = if target_width == 0 || target_height == 0 {
                0.0
            } else {
                1.0
            };
            w.set_opacity(target_op);
            if let Some(ref child) = w.first_child() {
                child.set_opacity(target_op);
            }
            if let Some(cb) = on_complete_opt.borrow_mut().take() {
                cb();
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed_us as f64 / dur_us as f64;
        let eased = Easing::EaseOutBackSoft.apply(t);
        let current_w = interpolate(start_width, target_width, eased);
        let current_h = interpolate(start_height, target_height, eased);
        w.set_size_request(current_w.max(0), current_h.max(0));

        if target_width == 0 || target_height == 0 {
            let opacity = (1.0 - eased).clamp(0.0, 1.0);
            w.set_opacity(opacity);
        } else {
            w.set_opacity(1.0);
        }

        glib::ControlFlow::Continue
    });
}
