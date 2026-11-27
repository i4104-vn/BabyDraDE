//! Slide-in widget animations.

use super::handler::{
    entering_margin, start_margin, AnimationGeneration, GenerationToken, SlideDirection,
};
use animation::Easing;
use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

/// Slides a widget into view, cancellable through a caller-owned counter.
///
/// Kept for existing callers; new code should use [`slide_in_generated`].
pub fn slide_in_cancelable(
    widget: &gtk4::Widget,
    direction: SlideDirection,
    distance_px: i32,
    duration_ms: u64,
    generation: Rc<Cell<u64>>,
    expected_generation: u64,
) {
    slide_in_generated(
        widget,
        direction,
        distance_px,
        duration_ms,
        &AnimationGeneration::from_cell(generation),
        GenerationToken::new(expected_generation),
    );
}

/// Slides a widget into view and restores its rest state when superseded.
pub fn slide_in_generated(
    widget: &gtk4::Widget,
    direction: SlideDirection,
    distance_px: i32,
    duration_ms: u64,
    generation: &AnimationGeneration,
    token: GenerationToken,
) {
    slide_in_inner(
        widget,
        direction,
        distance_px,
        duration_ms,
        Some((generation.clone(), token)),
    );
}

/// Slides a widget into view from the given direction using frame-synced FrameClock timing.
pub fn slide_in(
    widget: &gtk4::Widget,
    direction: SlideDirection,
    distance_px: i32,
    duration_ms: u64,
) {
    slide_in_inner(widget, direction, distance_px, duration_ms, None);
}

fn slide_in_inner(
    widget: &gtk4::Widget,
    direction: SlideDirection,
    distance_px: i32,
    duration_ms: u64,
    cancellation: Option<(AnimationGeneration, GenerationToken)>,
) {
    widget.set_opacity(0.0);
    widget.set_visible(true);

    let original_margin_top = widget.margin_top();
    let original_margin_bottom = widget.margin_bottom();
    let original_margin_start = widget.margin_start();
    let original_margin_end = widget.margin_end();

    match direction {
        SlideDirection::Down => {
            widget.set_margin_top(start_margin(original_margin_top, distance_px))
        }
        SlideDirection::Up => {
            widget.set_margin_bottom(start_margin(original_margin_bottom, distance_px))
        }
        SlideDirection::Right => {
            widget.set_margin_start(start_margin(original_margin_start, distance_px))
        }
        SlideDirection::Left => {
            widget.set_margin_end(start_margin(original_margin_end, distance_px))
        }
    }

    let start_time = std::cell::Cell::new(0i64);
    let dur_us = duration_ms as i64 * 1000;

    widget.add_tick_callback(move |w, clock| {
        if w.root().is_none() {
            return glib::ControlFlow::Break;
        }
        if let Some((ref generation, token)) = cancellation {
            if !generation.is_current(token) {
                // Never leave the widget invisible after a cancelled entry.
                w.set_opacity(1.0);
                match direction {
                    SlideDirection::Down => w.set_margin_top(original_margin_top),
                    SlideDirection::Up => w.set_margin_bottom(original_margin_bottom),
                    SlideDirection::Right => w.set_margin_start(original_margin_start),
                    SlideDirection::Left => w.set_margin_end(original_margin_end),
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
            w.set_opacity(1.0);
            match direction {
                SlideDirection::Down => w.set_margin_top(original_margin_top),
                SlideDirection::Up => w.set_margin_bottom(original_margin_bottom),
                SlideDirection::Right => w.set_margin_start(original_margin_start),
                SlideDirection::Left => w.set_margin_end(original_margin_end),
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed_us as f64 / dur_us as f64;
        let eased = Easing::EaseOutCubic.apply(t);

        w.set_opacity(eased.clamp(0.0, 1.0));
        match direction {
            SlideDirection::Down => {
                w.set_margin_top(entering_margin(original_margin_top, distance_px, eased))
            }
            SlideDirection::Up => {
                w.set_margin_bottom(entering_margin(original_margin_bottom, distance_px, eased))
            }
            SlideDirection::Right => {
                w.set_margin_start(entering_margin(original_margin_start, distance_px, eased))
            }
            SlideDirection::Left => {
                w.set_margin_end(entering_margin(original_margin_end, distance_px, eased))
            }
        }
        glib::ControlFlow::Continue
    });
}
