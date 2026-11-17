pub mod render;

use gtk4::glib;
use gtk4::prelude::*;
use services::wallpaper::get_wallpaper;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
    let started = Instant::now();
    let progress_for_draw = reveal_progress.clone();
    let center_for_draw = reveal_center.clone();
    let path_for_draw = wallpaper_path.clone();
    let ready_for_draw = wallpaper_ready.clone();
    drawing_area.set_draw_func(move |_, cr, width, height| {
        if !ready_for_draw.get() {
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
        if path_for_draw.borrow().is_none() {
            *path_for_draw.borrow_mut() = Some(get_wallpaper());
        }
        let path_ref = path_for_draw.borrow();
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

    let animation_area = drawing_area.clone();
    glib::timeout_add_local(Duration::from_millis(16), move || {
        let progress = (started.elapsed().as_secs_f64() / 0.7).clamp(0.0, 1.0);
        reveal_progress.set(progress);
        animation_area.queue_draw();
        if progress >= 1.0 {
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
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
