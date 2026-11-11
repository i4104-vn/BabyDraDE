//! Tile and Control components — Toggle tiles, square action tiles, and color buttons.

use crate::icons::{get_icon, get_icon_colored};
use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

/// Creates an interactive toggle tile (e.g. for Wi-Fi or Bluetooth in Control Center).
/// Returns the tile button and its dynamic subtitle label.
pub fn create_toggle_tile(
    icon_name: &str,
    title: &str,
    subtitle: &str,
    extra_css: &str,
    active: bool,
    on_toggle: impl Fn(bool) + 'static,
) -> (gtk4::Button, gtk4::Label) {
    let button = gtk4::Button::new();
    button.add_css_class("control-tile-left-btn");
    if !extra_css.is_empty() {
        for cls in extra_css.split_whitespace() {
            button.add_css_class(cls);
        }
    }
    if active {
        button.add_css_class("active");
    }

    let h_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    h_box.set_valign(gtk4::Align::Center);
    h_box.set_hexpand(true);

    let icon_color = if active {
        "#ffffff"
    } else {
        "rgba(255, 255, 255, 0.7)"
    };
    let icon = get_icon_colored(icon_name, 18, icon_color);
    icon.set_valign(gtk4::Align::Center);
    h_box.append(&icon);

    let v_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    v_box.set_valign(gtk4::Align::Center);
    v_box.set_hexpand(true);

    let title_label = gtk4::Label::new(Some(title));
    title_label.add_css_class("tile-title");
    title_label.set_xalign(0.0);
    title_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);

    let sub_label = gtk4::Label::new(Some(subtitle));
    sub_label.add_css_class("tile-subtitle");
    sub_label.set_xalign(0.0);
    sub_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);

    v_box.append(&title_label);
    v_box.append(&sub_label);
    h_box.append(&v_box);

    button.set_child(Some(&h_box));

    let active_state = Rc::new(Cell::new(active));
    let on_toggle_rc = Rc::new(on_toggle);
    let button_c = button.clone();
    let icon_name_c = icon_name.to_string();

    button.connect_clicked(move |_| {
        let new_state = !active_state.get();
        active_state.set(new_state);

        update_toggle_state(&button_c, new_state, &icon_name_c);
        on_toggle_rc(new_state);
    });

    (button, sub_label)
}

/// Updates the active appearance of a toggle tile.
pub fn update_toggle_state(btn: &gtk4::Button, active: bool, icon_name: &str) {
    if active {
        btn.add_css_class("active");
    } else {
        btn.remove_css_class("active");
    }

    let icon_color = if active {
        "#ffffff"
    } else {
        "rgba(255, 255, 255, 0.7)"
    };
    if let Some(child) = btn.child() {
        if let Some(h_box) = child.downcast_ref::<gtk4::Box>() {
            if let Some(first) = h_box.first_child() {
                if first.is::<gtk4::Image>() {
                    h_box.remove(&first);
                    let new_icon = get_icon_colored(icon_name, 18, icon_color);
                    new_icon.set_valign(gtk4::Align::Center);
                    h_box.prepend(&new_icon);
                }
            }
        }
    }
}

/// Creates a square quick toggle tile (56x56) for Control Center (DND, VPN, Night Light, Clean).
pub fn create_square_tile(
    icon_name: &str,
    label: &str,
    active: bool,
    on_toggle: impl Fn(bool) + 'static,
) -> gtk4::Button {
    let button = gtk4::Button::new();
    button.add_css_class("control-square-tile");
    button.set_size_request(56, 56);
    button.set_halign(gtk4::Align::Center);
    button.set_valign(gtk4::Align::Center);

    if active {
        button.add_css_class("active");
    }

    let icon_color = if active {
        "#ffffff"
    } else {
        "rgba(255, 255, 255, 0.8)"
    };
    let icon = get_icon_colored(icon_name, 18, icon_color);

    if label.is_empty() {
        button.set_child(Some(&icon));
    } else {
        let v_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        v_box.set_valign(gtk4::Align::Center);
        v_box.set_halign(gtk4::Align::Center);

        let lbl = gtk4::Label::new(Some(label));
        lbl.add_css_class("control-square-label");
        v_box.append(&icon);
        v_box.append(&lbl);
        button.set_child(Some(&v_box));
    }

    let active_state = Rc::new(Cell::new(active));
    let on_toggle_rc = Rc::new(on_toggle);
    let button_c = button.clone();
    let icon_name_c = icon_name.to_string();

    button.connect_clicked(move |_| {
        let new_state = !active_state.get();
        active_state.set(new_state);

        if new_state {
            button_c.add_css_class("active");
        } else {
            button_c.remove_css_class("active");
        }

        let col = if new_state {
            "#ffffff"
        } else {
            "rgba(255, 255, 255, 0.8)"
        };
        let new_icon = get_icon_colored(&icon_name_c, 18, col);
        button_c.set_child(Some(&new_icon));

        on_toggle_rc(new_state);
    });

    button
}

/// Creates a styled icon button with custom color and classes.
pub fn create_color_btn(
    icon_name: &str,
    size: i32,
    color: &str,
    classes: &[&str],
    tooltip: Option<&str>,
    on_click: impl Fn() + 'static,
) -> gtk4::Button {
    let button = gtk4::Button::new();
    for cls in classes {
        button.add_css_class(cls);
    }

    let icon = get_icon_colored(icon_name, size, color);
    button.set_child(Some(&icon));

    if let Some(tip) = tooltip {
        button.set_tooltip_text(Some(tip));
    }

    button.connect_clicked(move |_| {
        on_click();
    });

    button
}

/// Creates an icon button with default symbolic styling.
pub fn create_icon_button(
    icon_name: &str,
    size: i32,
    classes: &[&str],
    tooltip: Option<&str>,
    on_click: impl Fn() + 'static,
) -> gtk4::Button {
    let button = gtk4::Button::new();
    for cls in classes {
        button.add_css_class(cls);
    }

    let icon = get_icon(icon_name, size);
    button.set_child(Some(&icon));

    if let Some(tip) = tooltip {
        button.set_tooltip_text(Some(tip));
    }

    button.connect_clicked(move |_| {
        on_click();
    });

    button
}

/// Creates a Wi-Fi icon corresponding to signal strength (0-100%).
pub fn create_wifi_net_icon(
    signal: u8,
    _is_connected: bool,
    size: i32,
    color: Option<&str>,
) -> gtk4::Image {
    let icon_name = if signal > 75 {
        "network-wireless-signal-excellent-symbolic"
    } else if signal > 50 {
        "network-wireless-signal-good-symbolic"
    } else if signal > 25 {
        "network-wireless-signal-ok-symbolic"
    } else {
        "network-wireless-signal-weak-symbolic"
    };

    if let Some(c) = color {
        get_icon_colored(icon_name, size, c)
    } else {
        get_icon(icon_name, size)
    }
}
