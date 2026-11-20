//! Battery status indicator drawing area

use gtk4::glib;
use gtk4::prelude::*;
use services::system::BatteryReader;
use std::cell::RefCell;
use std::rc::Rc;
use ui_kit::drawing::draw_cairo_battery;
use ui_kit::icons::is_dark_mode;

pub fn create_battery_w() -> Option<gtk4::DrawingArea> {
    let mut reader = BatteryReader::new();
    let initial_info = reader.read()?;

    let drawing_area = gtk4::DrawingArea::new();
    drawing_area.set_content_width(24);
    drawing_area.set_content_height(14);
    drawing_area.set_valign(gtk4::Align::Center);
    drawing_area.set_halign(gtk4::Align::Center);

    let info_cache = Rc::new(RefCell::new(initial_info));
    let draw_cache = info_cache.clone();

    drawing_area.set_draw_func(move |_area, cr, width, height| {
        let info = draw_cache.borrow();
        let is_charging = matches!(info.status, services::system::BatteryStatus::Charging);
        let dark = is_dark_mode();
        draw_cairo_battery(
            cr,
            width as f64,
            height as f64,
            info.capacity as u32,
            is_charging,
            dark,
        );
    });

    let poll_cache = info_cache;
    let poll_area = drawing_area.clone();
    glib::timeout_add_seconds_local(10, move || {
        if let Some(info) = reader.read() {
            let changed = *poll_cache.borrow() != info;
            if changed {
                *poll_cache.borrow_mut() = info;
                poll_area.queue_draw();
            }
        }
        glib::ControlFlow::Continue
    });

    Some(drawing_area)
}
