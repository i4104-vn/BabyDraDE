pub mod render;

use gtk4::glib;
use gtk4::prelude::*;
use services::wallpaper::get_wallpaper;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

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
    let animation_started = Rc::new(Cell::new(None::<Instant>));
    let image_prepared = Rc::new(Cell::new(false));
    let progress_for_draw = reveal_progress.clone();
    let center_for_draw = reveal_center.clone();
    let path_for_draw = wallpaper_path.clone();
    let ready_for_draw = wallpaper_ready.clone();
    let animation_for_draw = animation_started.clone();
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
            animation_for_draw.set(Some(Instant::now()));
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

    // Smooth vsync-synced frame rendering via tick callback
    let animation_for_tick = animation_started.clone();
    let reveal_progress_tick = reveal_progress.clone();
    drawing_area.add_tick_callback(move |widget, _| {
        if let Some(started) = animation_for_tick.get() {
            let elapsed = started.elapsed().as_secs_f64() / 0.6;
            let linear_p = elapsed.clamp(0.0, 1.0);
            // Ease-out cubic: 1 - (1 - t)^3 for silky smooth reveal
            let progress = 1.0 - (1.0 - linear_p).powi(3);
            reveal_progress_tick.set(progress);
            widget.queue_draw();

            if linear_p >= 1.0 {
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
