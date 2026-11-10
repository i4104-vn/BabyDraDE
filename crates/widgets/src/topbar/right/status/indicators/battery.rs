//! Battery status indicator drawing area

use gtk4::prelude::*;
use services::system::BatteryReader;
use ui_kit::drawing::draw_cairo_battery;
use ui_kit::icons::is_dark_mode;

pub fn create_battery_w() -> Option<gtk4::DrawingArea> {
    let mut reader = BatteryReader::new();
    reader.read()?;

    let drawing_area = gtk4::DrawingArea::new();
    drawing_area.set_content_width(24);
    drawing_area.set_content_height(14);
    drawing_area.set_valign(gtk4::Align::Center);
    drawing_area.set_halign(gtk4::Align::Center);

    drawing_area.set_draw_func(move |_area, cr, width, height| {
        let mut r = BatteryReader::new();
        if let Some(info) = r.read() {
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
        }
    });

    Some(drawing_area)
}
