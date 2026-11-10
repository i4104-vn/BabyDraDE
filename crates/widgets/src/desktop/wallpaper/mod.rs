pub mod render;

use gtk4::prelude::*;
use services::wallpaper::get_wallpaper;
use std::cell::Cell;
use std::rc::Rc;

pub fn create_wallpaper_w() -> (gtk4::Overlay, Rc<dyn Fn(bool)>) {
    let container = gtk4::Overlay::new();
    container.set_hexpand(true);
    container.set_vexpand(true);
    container.add_css_class("desktop-wallpaper-container");

    let drawing_area = gtk4::DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);

    drawing_area.set_draw_func(move |_, cr, width, height| {
        let wp_path = get_wallpaper();
        render::render_wallpaper_background(cr, width as f64, height as f64, wp_path.as_deref());
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
