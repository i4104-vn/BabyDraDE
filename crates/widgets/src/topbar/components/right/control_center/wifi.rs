//! Wi-Fi Tile and Popover UI for Control Center

use gtk4::prelude::*;
use infrastructure::i18n::trans;
use infrastructure::system::network::{get_active_network_info, ActiveNetworkType};
use std::rc::Rc;
use ui_kit::components::{create_color_btn, create_popover, create_toggle_tile, update_toggle_state};
use ui_kit::icons::get_icon_colored;

/// Sets up the Wi-Fi popover listing active networks
fn setup_wifi_popover(popover: &gtk4::Popover) {
    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    main_box.add_css_class("wifi-popover-box");
    main_box.set_size_request(260, -1);
    main_box.set_margin_start(4);
    main_box.set_margin_end(4);

    let active_net = get_active_network_info();

    let title = gtk4::Label::new(Some(&trans("wifi.networks")));
    title.add_css_class("audio-menu-section-title");
    title.set_xalign(0.0);
    main_box.append(&title);

    if active_net.is_connected {
        let is_eth = active_net.network_type == ActiveNetworkType::Ethernet;
        let icon_name = if is_eth { "desktop" } else { "wifi" };

        let item_btn = gtk4::Button::new();
        item_btn.add_css_class("audio-menu-item-btn");
        item_btn.add_css_class("active");

        let item = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        item.set_valign(gtk4::Align::Center);

        let icon = get_icon_colored(icon_name, 14, "#ffffff");
        item.append(&icon);

        let labels = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        labels.set_hexpand(true);

        let name_lbl = gtk4::Label::new(Some(&active_net.name));
        name_lbl.set_halign(gtk4::Align::Start);
        labels.append(&name_lbl);

        let sub_lbl = gtk4::Label::new(Some(&format!(
            "{} • {}",
            trans("control.connected"),
            active_net.ip_address
        )));
        sub_lbl.set_halign(gtk4::Align::Start);
        sub_lbl.add_css_class("settings-row-desc");
        labels.append(&sub_lbl);

        item.append(&labels);

        let check = gtk4::Label::new(Some("✓"));
        check.add_css_class("audio-menu-item-check");
        item.append(&check);

        item_btn.set_child(Some(&item));
        main_box.append(&item_btn);
    } else {
        let no_net = gtk4::Label::new(Some(&trans("wifi.no_networks")));
        no_net.add_css_class("tile-subtitle");
        no_net.set_margin_top(10);
        no_net.set_margin_bottom(10);
        main_box.append(&no_net);
    }

    popover.set_child(Some(&main_box));
}

/// Creates a new Wi-Fi toggle card with arrow button to open network popover.
pub fn create_wifi_tile(on_popover_toggled: Option<Rc<dyn Fn(bool) + 'static>>) -> gtk4::Box {
    let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    container.add_css_class("control-tile-container");
    container.set_hexpand(false);

    let active_info = get_active_network_info();
    let initial_name = if active_info.is_connected {
        active_info.name.clone()
    } else {
        trans("control.off")
    };

    let (left_btn, sub_label) = create_toggle_tile(
        "wifi",
        &trans("control.network"),
        &initial_name,
        "control-tile-left-btn",
        active_info.is_connected,
        |_| {},
    );
    left_btn.set_hexpand(false);

    let right_btn = create_color_btn(
        "go-next-symbolic",
        12,
        "rgba(255, 255, 255, 0.7)",
        &["control-tile-right-btn"],
        None,
        || {},
    );

    let popover = create_popover(
        &container,
        gtk4::PositionType::Right,
        "wifi-popover control-popover",
    );

    setup_wifi_popover(&popover);

    let on_popover_toggled_c = on_popover_toggled.clone();
    let popover_c1 = popover.clone();
    let right_btn_c1 = right_btn.clone();
    right_btn.connect_clicked(move |_| {
        popover_c1.popup();
        if let Some(ref cb) = on_popover_toggled_c {
            cb(true);
        }
        let left_icon = get_icon_colored("go-previous-symbolic", 12, "rgba(255, 255, 255, 0.7)");
        right_btn_c1.set_child(Some(&left_icon));
    });

    let right_btn_c2 = right_btn.clone();
    let on_popover_toggled_c2 = on_popover_toggled;
    popover.connect_closed(move |_| {
        if let Some(ref cb) = on_popover_toggled_c2 {
            cb(false);
        }
        let right_icon = get_icon_colored("go-next-symbolic", 12, "rgba(255, 255, 255, 0.7)");
        right_btn_c2.set_child(Some(&right_icon));
    });

    let sub_lbl_c = sub_label;
    let left_btn_c = left_btn.clone();
    left_btn.connect_clicked(move |b| {
        let is_now_active = b.has_css_class("active");
        let active_net = get_active_network_info();
        let subtitle = if is_now_active {
            active_net.name
        } else {
            trans("control.off")
        };
        sub_lbl_c.set_text(&subtitle);
        update_toggle_state(&left_btn_c, is_now_active, "wifi");
    });

    container.append(&left_btn);
    container.append(&right_btn);

    container
}
