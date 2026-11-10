//! VPN Quick Tile & Popover UI for Control Center

use gtk4::prelude::*;
use services::i18n::trans;
use services::system::vpn::is_vpn_active;
use std::rc::Rc;
use ui_kit::components::{create_color_btn, create_popover};
use ui_kit::icons::get_icon_colored;

/// Sets up the VPN popover layout
fn setup_vpn_popover(popover: &gtk4::Popover) {
    let popover_box = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    popover_box.add_css_class("media-popover-box");
    popover_box.set_size_request(240, -1);
    popover_box.set_margin_start(4);
    popover_box.set_margin_end(4);

    let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    header.add_css_class("media-popover-header");

    let active = is_vpn_active();
    let icon_color = if active { "#10b981" } else { "rgba(255, 255, 255, 0.6)" };
    let icon = get_icon_colored("shield", 14, icon_color);
    let title = gtk4::Label::new(Some(&trans("vpn.title")));
    title.add_css_class("media-popover-app-name");

    header.append(&icon);
    header.append(&title);
    popover_box.append(&header);

    let status_str = if active {
        trans("control.connected")
    } else {
        trans("control.off")
    };
    let status_lbl = gtk4::Label::new(Some(&status_str));
    status_lbl.add_css_class("tile-subtitle");
    status_lbl.set_margin_top(8);
    status_lbl.set_margin_bottom(8);
    popover_box.append(&status_lbl);

    popover.set_child(Some(&popover_box));
}

/// Creates a new VPN square tile with popover.
pub fn create_vpn_tile(on_popover_toggled: Option<Rc<dyn Fn(bool) + 'static>>) -> gtk4::Button {
    let active = is_vpn_active();
    let initial_color = if active { "#ffffff" } else { "rgba(255, 255, 255, 0.8)" };

    let btn = create_color_btn(
        "shield",
        18,
        initial_color,
        &["control-square-tile"],
        None,
        || {},
    );
    btn.set_size_request(56, 56);
    btn.set_halign(gtk4::Align::Center);
    btn.set_valign(gtk4::Align::Center);

    if active {
        btn.add_css_class("active");
    }

    let popover = create_popover(
        &btn,
        gtk4::PositionType::Bottom,
        "vpn-popover media-popover control-popover",
    );

    setup_vpn_popover(&popover);

    let popover_c = popover.clone();
    let on_popover_toggled_c = on_popover_toggled.clone();
    btn.connect_clicked(move |_| {
        popover_c.popup();
        if let Some(ref cb) = on_popover_toggled_c {
            cb(true);
        }
    });

    let on_popover_toggled_c2 = on_popover_toggled;
    popover.connect_closed(move |_| {
        if let Some(ref cb) = on_popover_toggled_c2 {
            cb(false);
        }
    });

    btn
}

