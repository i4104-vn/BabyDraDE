//! Control Center Quick Toggle Grid Layout

use gtk4::prelude::*;
use std::process::Command;
use std::rc::Rc;
use ui_kit::components::create_square_tile;

use super::bluetooth::create_bt_tile;
use super::clean::create_clean_tile;
use super::vpn::create_vpn_tile;
use super::wifi::create_wifi_tile;

/// Checks if gammastep or wl-gammarelay is running
fn is_night_light_active() -> bool {
    let check = |name: &str| {
        Command::new("pgrep")
            .arg("-x")
            .arg(name)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };
    check("gammastep") || check("wl-gammarelay")
}

/// Creates a new Night Light quick tile
pub fn create_night_tile() -> gtk4::Button {
    let active = is_night_light_active();
    create_square_tile("night-light", "", active, |new_active| {
        if new_active {
            let _ = Command::new("gammastep")
                .args(["-O", "4500", "-b", "1.0:1.0"])
                .spawn();
        } else {
            let _ = Command::new("pkill").arg("-x").arg("gammastep").status();
        }
    })
}

/// Creates a new DND quick tile
pub fn create_dnd_tile() -> gtk4::Button {
    create_square_tile("bell-off", "", false, |_new_active| {})
}

/// Creates the complete Control Center toggle grid (left stack + right 2x2 grid).
pub fn create_cc_grid(on_popover_toggled: Option<Rc<dyn Fn(bool) + 'static>>) -> gtk4::Box {
    let main_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    main_layout.add_css_class("control-center-grid");
    main_layout.set_hexpand(true);
    main_layout.set_valign(gtk4::Align::Fill);
    main_layout.set_vexpand(true);

    // Left column: Wi-Fi + Bluetooth
    let left_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    left_box.add_css_class("control-left-toggles-box");
    left_box.set_valign(gtk4::Align::Fill);
    left_box.set_vexpand(true);

    let wifi_tile = create_wifi_tile(on_popover_toggled.clone());
    let bt_btn = create_bt_tile();

    left_box.append(&wifi_tile);
    left_box.append(&bt_btn);

    // Right 2x2 grid: DND & VPN (top), Clean & Night Light (bottom)
    let right_grid = gtk4::Grid::new();
    right_grid.set_column_spacing(10);
    right_grid.set_row_spacing(10);
    right_grid.set_hexpand(true);
    right_grid.set_vexpand(true);
    right_grid.set_valign(gtk4::Align::Fill);

    let top_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    top_box.set_hexpand(true);
    top_box.set_homogeneous(true);
    top_box.set_vexpand(true);
    top_box.set_valign(gtk4::Align::Fill);

    let dnd_btn = create_dnd_tile();
    let vpn_btn = create_vpn_tile(on_popover_toggled.clone());
    top_box.append(&dnd_btn);
    top_box.append(&vpn_btn);
    right_grid.attach(&top_box, 0, 0, 2, 1);

    let bottom_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    bottom_box.set_hexpand(true);
    bottom_box.set_homogeneous(true);
    bottom_box.set_vexpand(true);
    bottom_box.set_valign(gtk4::Align::Fill);

    let clean_btn = create_clean_tile(on_popover_toggled);
    let night_btn = create_night_tile();

    bottom_box.append(&clean_btn);
    bottom_box.append(&night_btn);
    right_grid.attach(&bottom_box, 0, 1, 2, 1);

    main_layout.append(&left_box);
    main_layout.append(&right_grid);

    main_layout
}
