pub mod render;

use animation::{Easing, Tween, REVEAL_DURATION};
use gtk4::glib;
use gtk4::prelude::*;
use services::wallpaper::get_wallpaper;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Handle that pauses (`true`) or resumes (`false`) the wallpaper reveal.
#[derive(Clone)]
pub struct WallpaperFreeze(Rc<dyn Fn(bool)>);

impl WallpaperFreeze {
    /// Sets whether the reveal animation is frozen.
    pub fn set(&self, frozen: bool) {
        (self.0)(frozen);
    }
}

/// Decides if a freshly read config path must replace the displayed one.
///
/// The first load must always go through, even when both paths are `None`
/// (i.e. the embedded default wallpaper is in use).
fn needs_apply(ready: bool, current: &Option<PathBuf>, next: &Option<PathBuf>) -> bool {
    !ready || current != next
}

/// Builds the background wallpaper widget and returns it with its freeze handle.
///
/// # Returns
/// The layer-shell background overlay plus a handle that pauses/resumes its
/// reveal animation.
pub fn create_wallpaper_w() -> (gtk4::Overlay, WallpaperFreeze) {
    let container = gtk4::Overlay::new();
    container.set_hexpand(true);
    container.set_vexpand(true);
    container.add_css_class("desktop-wallpaper-container");

    let drawing_area = gtk4::DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);

    let reveal_progress = Rc::new(Cell::new(0.0_f64));
    let reveal_center = Rc::new(Cell::new(None::<(f64, f64)>));
    let wallpaper_path = Rc::new(RefCell::new(None::<PathBuf>));
    let wallpaper_ready = Rc::new(Cell::new(false));
    let reveal_tween = Rc::new(Cell::new(None::<Tween>));
    let image_prepared = Rc::new(Cell::new(false));
    let frozen = Rc::new(Cell::new(false));
    let progress_for_draw = reveal_progress.clone();
    let center_for_draw = reveal_center.clone();
    let path_for_draw = wallpaper_path.clone();
    let ready_for_draw = wallpaper_ready.clone();
    let tween_for_draw = reveal_tween.clone();
    let prepared_for_draw = image_prepared.clone();
    let frozen_for_draw = frozen.clone();
    drawing_area.set_draw_func(move |_, cr, width, height| {
        if frozen_for_draw.get() {
            return;
        }
        if !ready_for_draw.get() {
            cr.set_source_rgb(0.02, 0.025, 0.04);
            let _ = cr.paint();
            return;
        }
        let path_ref = path_for_draw.borrow();
        if !prepared_for_draw.get() {
            render::prepare_wallpaper_background(width as f64, height as f64, path_ref.as_deref());
            prepared_for_draw.set(true);
            tween_for_draw.set(Some(Tween::new(
                0.0,
                1.0,
                REVEAL_DURATION,
                Easing::EaseInOutCubic,
            )));
            cr.set_source_rgb(0.02, 0.025, 0.04);
            let _ = cr.paint();
            return;
        }
        let center = center_for_draw.get().unwrap_or_else(|| {
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(1);
            let x = (seed % width.max(1) as u64) as f64;
            let y = (seed.rotate_left(17) % height.max(1) as u64) as f64;
            let value = (x, y);
            center_for_draw.set(Some(value));
            value
        });
        let progress = progress_for_draw.get();
        render::render_wallpaper_reveal(
            cr,
            width as f64,
            height as f64,
            path_ref.as_deref(),
            center,
            progress,
        );
    });

    // GTK tick callbacks are vsync-synced, so on a 120 Hz output this runs at
    // 120 fps. The Tween is wall-clock based, giving identical motion at any
    // refresh rate.
    let tween_for_tick = reveal_tween.clone();
    let reveal_progress_tick = reveal_progress.clone();
    let frozen_for_tick = frozen.clone();
    drawing_area.add_tick_callback(move |widget, _| {
        if frozen_for_tick.get() {
            return glib::ControlFlow::Continue;
        }
        if let Some(tween) = tween_for_tick.get() {
            reveal_progress_tick.set(tween.value());
            widget.queue_draw();

            if !tween.is_animating() {
                reveal_progress_tick.set(1.0);
                widget.queue_draw();
                return glib::ControlFlow::Break;
            }
        }
        glib::ControlFlow::Continue
    });

    let path_for_apply = wallpaper_path.clone();
    let ready_for_apply = wallpaper_ready.clone();
    let tween_for_apply = reveal_tween.clone();
    let center_for_apply = reveal_center.clone();
    let progress_for_apply = reveal_progress.clone();
    let prepared_for_apply = image_prepared.clone();
    let draw_for_apply = drawing_area.clone();
    let apply_wallpaper = Rc::new(move || {
        let next = get_wallpaper();
        let mut current = path_for_apply.borrow_mut();
        if !needs_apply(ready_for_apply.get(), &current, &next) {
            return;
        }
        let was_ready = ready_for_apply.get();
        *current = next;
        drop(current);

        if was_ready {
            // Win+F5 swaps instantly; only the first load gets the reveal
            // animation. Drop any in-flight tween so it cannot overwrite the
            // fully revealed frame.
            tween_for_apply.set(None);
            center_for_apply.set(None);
            progress_for_apply.set(1.0);
            prepared_for_apply.set(true);
        }
        ready_for_apply.set(true);
        draw_for_apply.queue_draw();
    });

    // First lookup runs after the layer-shell surface maps so the initial
    // paint is immediate.
    let idle_apply = apply_wallpaper.clone();
    glib::idle_add_local_once(move || idle_apply());

    // Win+F5 is a compositor shortcut. The bus delivers its reload event here;
    // polling the client inbox is the same pattern the topbar uses.
    let reload_apply = apply_wallpaper.clone();
    glib::timeout_add_local(Duration::from_millis(100), move || {
        if crate::bridge::take_reload() {
            reload_apply();
        }
        glib::ControlFlow::Continue
    });

    container.set_child(Some(&drawing_area));

    let draw_freeze = drawing_area.clone();
    let set_frozen = Rc::new(move |should_freeze| {
        frozen.set(should_freeze);
        if !should_freeze {
            draw_freeze.queue_draw();
        }
    });

    (container, WallpaperFreeze(set_frozen))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_apply_when_first_load_without_configured_path() {
        assert!(needs_apply(false, &None, &None));
    }

    #[test]
    fn should_skip_when_config_path_is_unchanged() {
        let path = Some(PathBuf::from("/home/user/.babydra/wallpaper/a.png"));
        assert!(!needs_apply(true, &path, &path));
    }

    #[test]
    fn should_apply_when_config_path_changed() {
        let old = Some(PathBuf::from("/home/user/.babydra/wallpaper/a.png"));
        let new = Some(PathBuf::from("/home/user/.babydra/wallpaper/b.png"));
        assert!(needs_apply(true, &old, &new));
    }
}
