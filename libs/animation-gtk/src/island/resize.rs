//! Dynamic Island size animations.

//! Widget-side rendering for Dynamic Island capsule transitions.

use std::cell::Cell;

use super::gen::{current_anim_gen, next_anim_gen};
use super::handler::interpolate;
use animation::Easing;
use gtk4::prelude::*;

thread_local! {
    static CURRENT_ANIM_GEN: Cell<u64> = const { Cell::new(0) };
}

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
