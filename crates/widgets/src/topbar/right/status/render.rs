//! Layout assembly for the status indicators row

use gtk4::prelude::*;
use services::system::{get_volume_state, is_vpn_active};
use ui_kit::icons::get_icon;

use super::indicators::{create_battery_w, create_network_widget};

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
