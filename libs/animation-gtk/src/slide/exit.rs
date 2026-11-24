//! Slide-out widget animations.

use super::handler::{exiting_margin, SlideDirection};
use animation::Easing;
use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

/// Slides a widget out of view using frame-synced FrameClock timing.
pub fn slide_out(
    widget: &gtk4::Widget,
    direction: SlideDirection,
    distance_px: i32,
    duration_ms: u64,
    hide_after: bool,
) {
    let start_opacity = widget.opacity();
    let original_margin_top = widget.margin_top();
    let original_margin_bottom = widget.margin_bottom();
    let original_margin_start = widget.margin_start();
    let original_margin_end = widget.margin_end();

    let start_time = std::cell::Cell::new(0i64);
    let dur_us = duration_ms as i64 * 1000;

    widget.add_tick_callback(move |w, clock| {
        if w.root().is_none() {
            return glib::ControlFlow::Break;
        }
        let now = clock.frame_time();
        if start_time.get() == 0 {
            start_time.set(now);
        }
        let elapsed_us = now - start_time.get();
        if elapsed_us >= dur_us {
            w.set_opacity(0.0);
            match direction {
                SlideDirection::Down => w.set_margin_bottom(original_margin_bottom),
                SlideDirection::Up => w.set_margin_top(original_margin_top),
                SlideDirection::Right => w.set_margin_end(original_margin_end),
                SlideDirection::Left => w.set_margin_start(original_margin_start),
            }
            if hide_after {
                w.set_visible(false);
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed_us as f64 / dur_us as f64;
        let eased = Easing::EaseOutCubic.apply(t);

        w.set_opacity((start_opacity * (1.0 - eased)).clamp(0.0, 1.0));
        match direction {
            SlideDirection::Down => {
                w.set_margin_bottom(exiting_margin(original_margin_bottom, distance_px, eased))
            }
            SlideDirection::Up => {
                w.set_margin_top(exiting_margin(original_margin_top, distance_px, eased))
            }
            SlideDirection::Right => {
                w.set_margin_end(exiting_margin(original_margin_end, distance_px, eased))
            }
            SlideDirection::Left => {
                w.set_margin_start(exiting_margin(original_margin_start, distance_px, eased))
            }
        }
        glib::ControlFlow::Continue
    });
}

/// Slides a widget out of view with a completion callback using frame-synced timing.
pub fn slide_out_cb<F>(
    widget: &gtk4::Widget,
    direction: SlideDirection,
    distance_px: i32,
    duration_ms: u64,
    hide_after: bool,
    on_complete: F,
) where
    F: FnOnce() + 'static,
{
    slide_out_cb_inner(
        widget,
        direction,
        distance_px,
        duration_ms,
        hide_after,
        None,
        on_complete,
    );
}

/// Slides out a widget and stops cleanly when a newer animation generation is started.
#[allow(clippy::too_many_arguments)]
pub fn slide_out_cb_cancelable<F>(
    widget: &gtk4::Widget,
    direction: SlideDirection,
    distance_px: i32,
    duration_ms: u64,
    hide_after: bool,
    generation: Rc<Cell<u64>>,
    expected_generation: u64,
    on_complete: F,
) where
    F: FnOnce() + 'static,
{
    slide_out_cb_inner(
        widget,
        direction,
        distance_px,
        duration_ms,
        hide_after,
        Some((generation, expected_generation)),
        on_complete,
    );
}

fn slide_out_cb_inner<F>(
    widget: &gtk4::Widget,
    direction: SlideDirection,
    distance_px: i32,
    duration_ms: u64,
    hide_after: bool,
    cancellation: Option<(Rc<Cell<u64>>, u64)>,
    on_complete: F,
) where
    F: FnOnce() + 'static,
{
    let start_opacity = widget.opacity();
    let original_margin_top = widget.margin_top();
    let original_margin_bottom = widget.margin_bottom();
    let original_margin_start = widget.margin_start();
    let original_margin_end = widget.margin_end();

    let start_time = std::cell::Cell::new(0i64);
    let dur_us = duration_ms as i64 * 1000;

    let on_complete_opt = std::cell::RefCell::new(Some(on_complete));

    widget.add_tick_callback(move |w, clock| {
        if w.root().is_none() {
            return glib::ControlFlow::Break;
        }
        if let Some((ref generation, expected)) = cancellation {
            if generation.get() != expected {
                match direction {
                    SlideDirection::Down => w.set_margin_bottom(original_margin_bottom),
                    SlideDirection::Up => w.set_margin_top(original_margin_top),
                    SlideDirection::Right => w.set_margin_end(original_margin_end),
                    SlideDirection::Left => w.set_margin_start(original_margin_start),
                }
                return glib::ControlFlow::Break;
            }
        }
        let now = clock.frame_time();
        if start_time.get() == 0 {
            start_time.set(now);
        }
        let elapsed_us = now - start_time.get();
        if elapsed_us >= dur_us {
            w.set_opacity(0.0);
            match direction {
                SlideDirection::Down => w.set_margin_bottom(original_margin_bottom),
                SlideDirection::Up => w.set_margin_top(original_margin_top),
                SlideDirection::Right => w.set_margin_end(original_margin_end),
                SlideDirection::Left => w.set_margin_start(original_margin_start),
            }
            if hide_after {
                w.set_visible(false);
            }
            if let Some(cb) = on_complete_opt.borrow_mut().take() {
                cb();
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed_us as f64 / dur_us as f64;
        let eased = Easing::EaseOutCubic.apply(t);

        w.set_opacity((start_opacity * (1.0 - eased)).clamp(0.0, 1.0));
        match direction {
            SlideDirection::Down => {
                w.set_margin_bottom(exiting_margin(original_margin_bottom, distance_px, eased))
            }
            SlideDirection::Up => {
                w.set_margin_top(exiting_margin(original_margin_top, distance_px, eased))
            }
            SlideDirection::Right => {
                w.set_margin_end(exiting_margin(original_margin_end, distance_px, eased))
            }
            SlideDirection::Left => {
                w.set_margin_start(exiting_margin(original_margin_start, distance_px, eased))
            }
        }
        glib::ControlFlow::Continue
    });
}
