//! Network status indicator widget (Wi-Fi icon / Ethernet Cairo drawing with 2s polling)

use gtk4::glib;
use gtk4::prelude::*;
use infrastructure::system::{get_active_network_info, ActiveNetworkType};
use std::cell::Cell;
use std::rc::Rc;
use ui_kit::drawing::draw_cairo_ethernet_desktop;
use ui_kit::icons::{get_icon, is_dark_mode, set_image_from_icon};

pub fn create_network_widget() -> gtk4::Box {
    let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    container.set_valign(gtk4::Align::Center);
    container.set_halign(gtk4::Align::Center);

    let wifi_icon = get_icon("wifi", 14);
    wifi_icon.set_valign(gtk4::Align::Center);
    wifi_icon.set_halign(gtk4::Align::Center);

    let eth_area = gtk4::DrawingArea::new();
    eth_area.set_content_width(16);
    eth_area.set_content_height(14);
    eth_area.set_valign(gtk4::Align::Center);
    eth_area.set_halign(gtk4::Align::Center);
    eth_area.set_visible(false);

    container.append(&wifi_icon);
    container.append(&eth_area);

    let is_connected_cell = Rc::new(Cell::new(true));
    let conn_draw = is_connected_cell.clone();

    eth_area.set_draw_func(move |_, cr, width, height| {
        let dark = is_dark_mode();
        draw_cairo_ethernet_desktop(
            cr,
            width as f64,
            height as f64,
            dark,
            conn_draw.get(),
            true,
            true,
        );
    });

    let eth_timer = eth_area.clone();
    let wifi_timer = wifi_icon.clone();
    let conn_timer = is_connected_cell.clone();

    glib::timeout_add_seconds_local(2, move || {
        let net = get_active_network_info();
        let is_eth = net.network_type == ActiveNetworkType::Ethernet;

        if is_eth {
            wifi_timer.set_visible(false);
            eth_timer.set_visible(true);
            conn_timer.set(net.is_connected);
            eth_timer.queue_draw();
        } else {
            eth_timer.set_visible(false);
            wifi_timer.set_visible(true);
            set_image_from_icon(&wifi_timer, "wifi", 14);
            wifi_timer.set_opacity(if net.is_connected { 1.0 } else { 0.5 });
        }

        glib::ControlFlow::Continue
    });

    container
}
