//! Progressive blur lock/unlock transition scheduling.

use animation::{Easing, Tween};
use gtk4::prelude::*;
use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

const BLUR_DIVISORS: [i32; 3] = [8, 16, 28];

pub(crate) use crate::modules::shell::lock::models::LockTransition;

pub(crate) fn reverse_lock_transition(stacks: Vec<LockTransition>, on_complete: Rc<dyn Fn()>) {
    if stacks.is_empty() {
        on_complete();
        return;
    }

    let remaining = Rc::new(Cell::new(stacks.len()));
    for transition in stacks {
        let remaining = remaining.clone();
        let on_complete = on_complete.clone();
        transition.cancelled.set(true);
        transition.window.set_opacity(1.0);
        transition.stack.set_opacity(1.0);
        transition.stack.set_visible(true);
        transition
            .stack
            .set_visible_child_name(&format!("blur-{}", transition.last_blur_frame.get()));
        schedule_reverse_blur_step(
            transition,
            Rc::new(move || {
                let count = remaining.get().saturating_sub(1);
                remaining.set(count);
                if count == 0 {
                    on_complete();
                }
            }),
        );
    }
}

fn schedule_reverse_blur_step(transition: LockTransition, on_complete: Rc<dyn Fn()>) {
    let current = transition.last_blur_frame.get();
    if current == 0 {
        fade_out_reverse_transition(transition, on_complete);
        return;
    }

    glib::timeout_add_local_once(Duration::from_millis(35), move || {
        let next = current.saturating_sub(1);
        transition
            .stack
            .set_visible_child_name(&format!("blur-{next}"));
        transition.last_blur_frame.set(next);
        if next == 0 {
            fade_out_reverse_transition(transition, on_complete);
        } else {
            schedule_reverse_blur_step(transition, on_complete);
        }
    });
}

fn fade_out_reverse_transition(transition: LockTransition, on_complete: Rc<dyn Fn()>) {
    let fade = Tween::new(1.0, 0.0, Duration::from_millis(200), Easing::EaseInOutCubic);
    // Tick callbacks follow the display refresh (up to 120 fps) instead of a
    // fixed timer, while the Tween keeps motion wall-clock based.
    transition.window.clone().add_tick_callback(move |_, _| {
        transition.window.set_opacity(fade.value());

        if !fade.is_animating() {
            transition.window.set_opacity(0.0);
            transition.window.set_visible(false);
            on_complete();
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}

pub(super) fn schedule_blur_transition(transition: LockTransition, image_path: PathBuf) {
    let source = match gtk4::gdk_pixbuf::Pixbuf::from_file(&image_path) {
        Ok(pixbuf) => pixbuf,
        Err(_) => return,
    };
    schedule_blur_step(transition, source, 0);
}

fn schedule_blur_step(transition: LockTransition, source: gtk4::gdk_pixbuf::Pixbuf, step: usize) {
    let delay = if step == 0 { 10 } else { 70 };

    glib::timeout_add_local_once(Duration::from_millis(delay), move || {
        if transition.cancelled.get() {
            return;
        }

        let divisor = BLUR_DIVISORS[step];
        if let Some(frame) = blur_transition_frame(&source, divisor) {
            let texture = gtk4::gdk::Texture::for_pixbuf(&frame);
            let picture = gtk4::Picture::new();
            picture.set_paintable(Some(&texture));
            picture.set_can_shrink(true);
            picture.set_hexpand(true);
            picture.set_vexpand(true);
            transition
                .stack
                .add_named(&picture, Some(&format!("blur-{}", step + 1)));
            transition
                .stack
                .set_visible_child_name(&format!("blur-{}", step + 1));
            transition.last_blur_frame.set(step + 1);
        }

        if step + 1 < BLUR_DIVISORS.len() {
            schedule_blur_step(transition, source, step + 1);
        } else {
            glib::timeout_add_local_once(Duration::from_millis(70), move || {
                fade_out_transition(transition);
            });
        }
    });
}

fn blur_transition_frame(
    source: &gtk4::gdk_pixbuf::Pixbuf,
    divisor: i32,
) -> Option<gtk4::gdk_pixbuf::Pixbuf> {
    let width = source.width();
    let height = source.height();
    let reduced = source.scale_simple(
        (width / divisor).max(1),
        (height / divisor).max(1),
        gtk4::gdk_pixbuf::InterpType::Bilinear,
    )?;
    reduced.scale_simple(width, height, gtk4::gdk_pixbuf::InterpType::Bilinear)
}

fn fade_out_transition(transition: LockTransition) {
    let fade = Tween::new(1.0, 0.0, Duration::from_millis(240), Easing::EaseInOutCubic);
    // Vsynced fade: updates once per display frame (up to 120 fps).
    transition.stack.clone().add_tick_callback(move |_, _| {
        if transition.cancelled.get() {
            return glib::ControlFlow::Break;
        }

        transition.stack.set_opacity(fade.value());

        if !fade.is_animating() {
            transition.stack.set_opacity(0.0);
            transition.stack.set_visible(false);
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}
