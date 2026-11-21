pub mod render;

use animation::{Easing, Tween, REVEAL_DURATION};
use gtk4::glib;
use gtk4::prelude::*;
use services::wallpaper::get_wallpaper;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn create_wallpaper_w() -> (gtk4::Overlay, Rc<dyn Fn(bool)>) {
    let container = gtk4::Overlay::new();
    container.set_hexpand(true);
    container.set_vexpand(true);
    container.add_css_class("desktop-wallpaper-container");

    let drawing_area = gtk4::DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);

    let reveal_progress = Rc::new(Cell::new(0.0_f64));
    let reveal_center = Rc::new(Cell::new(None::<(f64, f64)>));
    let wallpaper_path = Rc::new(RefCell::new(None::<Option<std::path::PathBuf>>));
    let wallpaper_ready = Rc::new(Cell::new(false));
    let reveal_tween = Rc::new(Cell::new(None::<Tween>));
    let image_prepared = Rc::new(Cell::new(false));
    let progress_for_draw = reveal_progress.clone();
    let center_for_draw = reveal_center.clone();
    let path_for_draw = wallpaper_path.clone();
    let ready_for_draw = wallpaper_ready.clone();
    let tween_for_draw = reveal_tween.clone();
    let prepared_for_draw = image_prepared.clone();
    drawing_area.set_draw_func(move |_, cr, width, height| {
        if !ready_for_draw.get() {
            cr.set_source_rgb(0.02, 0.025, 0.04);
            let _ = cr.paint();
            return;
        }
        let path_ref = path_for_draw.borrow();
        if !prepared_for_draw.get() {
            render::prepare_wallpaper_background(
                width as f64,
                height as f64,
                path_ref.as_ref().and_then(|p| p.as_deref()),
            );
            prepared_for_draw.set(true);
            tween_for_draw.set(Some(Tween::new(
                0.0,
                1.0,
                REVEAL_DURATION,
                Easing::EaseOutCubic,
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
            path_ref.as_ref().and_then(|p| p.as_deref()),
            center,
            progress,
        );
    });

    // Do config lookup after the layer-shell surface has had a chance to map;
    // the first paint is therefore immediate instead of waiting on disk IO.
    let load_path = wallpaper_path.clone();
    let load_ready = wallpaper_ready.clone();
    let load_area = drawing_area.clone();
    glib::idle_add_local_once(move || {
        *load_path.borrow_mut() = Some(get_wallpaper());
        load_ready.set(true);
        load_area.queue_draw();
    });

    // GTK tick callbacks are vsync-synced, so on a 120 Hz output this runs at
    // 120 fps. The Tween is wall-clock based, giving identical motion at any
    // refresh rate.
    let tween_for_tick = reveal_tween.clone();
    let reveal_progress_tick = reveal_progress.clone();
    drawing_area.add_tick_callback(move |widget, _| {
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

    container.set_child(Some(&drawing_area));

    let frozen = Rc::new(Cell::new(false));
    let draw_freeze = drawing_area.clone();
    let set_frozen = Rc::new(move |should_freeze| {
        frozen.set(should_freeze);
        if !should_freeze {
            draw_freeze.queue_draw();
        }
    });

    (container, set_frozen)
}
