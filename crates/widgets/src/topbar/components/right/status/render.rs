use gtk4::glib;
use gtk4::prelude::*;
use infrastructure::system::{get_active_network_info, get_volume_state, is_vpn_active, ActiveNetworkType, BatteryReader};
use std::cell::Cell;
use std::rc::Rc;
use ui_kit::drawing::{draw_cairo_battery, draw_cairo_ethernet_desktop};
use ui_kit::icons::{get_icon, is_dark_mode, set_image_from_icon};

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
            let is_charging = matches!(info.status, infrastructure::system::BatteryStatus::Charging);
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

pub fn build_status_row() -> (
    gtk4::Box,
    gtk4::Button,
    gtk4::Label,
    gtk4::Image,
    gtk4::Box,
    gtk4::Image,
    Option<gtk4::DrawingArea>,
) {
    let status_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    status_box.add_css_class("status-indicators-box");

    let status_button = gtk4::Button::new();
    status_button.add_css_class("panel-status-btn");

    let inner_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);

    let vpn_icon = get_icon("shield", 14);
    vpn_icon.add_css_class("status-icon");
    vpn_icon.set_visible(is_vpn_active());

    let net_container = create_network_widget();
    net_container.add_css_class("status-icon");

    let (_vol, is_m) = get_volume_state();
    let vol_icon = if is_m {
        get_icon("volume-mute", 14)
    } else {
        get_icon("volume", 14)
    };
    vol_icon.add_css_class("status-icon");

    inner_layout.append(&vpn_icon);
    inner_layout.append(&net_container);
    inner_layout.append(&vol_icon);

    let bat_widget = create_battery_w();
    if let Some(ref bat_area) = bat_widget {
        bat_area.add_css_class("status-icon");
        inner_layout.append(bat_area);
    }

    status_button.set_child(Some(&inner_layout));

    let separator = gtk4::Label::new(Some("│"));
    separator.add_css_class("capsule-separator");

    (
        status_box,
        status_button,
        separator,
        vol_icon,
        net_container,
        vpn_icon,
        bat_widget,
    )
}
