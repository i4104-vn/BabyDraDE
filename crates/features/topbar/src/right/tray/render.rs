use gtk4::prelude::*;
use services::tray::TrayItem;
use ui_kit::icons::get_theme_or_svg_icon;

const TRAY_ICON_SIZE: i32 = 16;

pub fn build_tray_container() -> gtk4::Box {
    let tray_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    tray_container.add_css_class("panel-tray-box");
    tray_container.set_valign(gtk4::Align::Center);
    tray_container.set_halign(gtk4::Align::Center);
    tray_container.set_visible(false);
    tray_container
}

pub fn build_tray_button(item: &TrayItem) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("panel-tray-item-btn");

    // Hover always shows a name, even for items that only advertise an id.
    let tooltip = if item.title.trim().is_empty() {
        if item.icon_name.is_empty() {
            &item.service
        } else {
            &item.icon_name
        }
    } else {
        &item.title
    };
    btn.set_tooltip_text(Some(tooltip));

    btn.set_valign(gtk4::Align::Center);
    btn.set_halign(gtk4::Align::Center);
    btn.set_receives_default(false);
    btn.set_focusable(false);
    btn.set_child(Some(&get_theme_or_svg_icon(
        &item.icon_name,
        TRAY_ICON_SIZE,
    )));

    btn
}
