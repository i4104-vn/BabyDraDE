//! Brightness Slider UI item for Control Center

use gtk4::prelude::*;
use infrastructure::i18n::trans;
use infrastructure::system::hardware::backlight::BacklightReader;
use std::cell::RefCell;
use std::rc::Rc;
use ui_kit::components::{create_slider_header, PillSlider};
use ui_kit::icons::get_icon_colored;

/// Returns the symbolic icon name for a given brightness percentage.
fn get_brightness_icon_name(val: f64) -> &'static str {
    if val <= 20.0 {
        "brightness-low"
    } else if val <= 70.0 {
        "brightness-medium"
    } else {
        "brightness"
    }
}

pub fn get_brightness() -> f64 {
    BacklightReader::new()
        .read()
        .map(|b| b.percent as f64)
        .unwrap_or(80.0)
}

pub fn set_brightness(val: f64) {
    let mut reader = BacklightReader::new();
    let _ = reader.set_brightness_percent(val.clamp(1.0, 100.0).round() as u8);
}

/// Creates a new brightness control row.
pub fn create_brightness() -> (gtk4::Box, Rc<dyn Fn(f64)>) {
    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    main_box.add_css_class("control-slider-card");

    let initial_val = get_brightness();
    let (header_box, value_label) = create_slider_header(&trans("common.brightness"), initial_val);

    let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

    let icon_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    icon_container.set_valign(gtk4::Align::Center);
    icon_container.set_halign(gtk4::Align::Start);
    icon_container.set_margin_start(10);

    let current_icon_name = Rc::new(RefCell::new(get_brightness_icon_name(initial_val)));
    let update_icon = {
        let icon_container = icon_container.clone();
        let current_icon_name = current_icon_name.clone();
        Rc::new(move |val: f64| {
            let new_name = get_brightness_icon_name(val);
            if *current_icon_name.borrow() != new_name {
                *current_icon_name.borrow_mut() = new_name;
                if let Some(old) = icon_container.first_child() {
                    icon_container.remove(&old);
                }
                let icon_widget = get_icon_colored(new_name, 16, "#ffffff");
                icon_widget.add_css_class("slider-overlay-icon");
                icon_container.append(&icon_widget);
            }
        })
    };

    let initial_icon = get_icon_colored(get_brightness_icon_name(initial_val), 16, "#ffffff");
    initial_icon.add_css_class("slider-overlay-icon");
    icon_container.append(&initial_icon);

    let update_icon_drag = update_icon.clone();
    let slider = PillSlider::new_range(1.0, 100.0, 1.0, initial_val, move |val| {
        update_icon_drag(val);
    });
    slider.bind_label(&value_label);
    slider.add_scroll_to(&row_box);
    slider.add_scroll_to(&main_box);

    let update_icon_debounced = update_icon.clone();
    slider.connect_debounced(80, move |val| {
        set_brightness(val);
        update_icon_debounced(val);
    });

    let overlay = gtk4::Overlay::new();
    overlay.set_hexpand(true);
    overlay.set_valign(gtk4::Align::Center);
    overlay.set_child(Some(&slider.container));
    overlay.add_overlay(&icon_container);

    row_box.append(&overlay);

    main_box.append(&header_box);
    main_box.append(&row_box);

    let slider_sync = slider.clone();
    let update_icon_sync = update_icon.clone();
    let sync_callback: Rc<dyn Fn(f64)> = Rc::new(move |val: f64| {
        slider_sync.set_value_silent(val);
        update_icon_sync(val);
    });

    (main_box, sync_callback)
}

