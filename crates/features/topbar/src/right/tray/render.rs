use gtk4::prelude::*;
use services::tray::TrayItem;
use std::path::Path;
use ui_kit::icons::get_theme_or_svg_icon;

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
    btn.set_tooltip_text(Some(&item.title));
    btn.set_valign(gtk4::Align::Center);
    btn.set_halign(gtk4::Align::Center);
    btn.set_receives_default(false);
    btn.set_focusable(false);
    let icon = resolve_icon(&item.icon_name, &item.icon_theme_path);
    btn.set_child(Some(&icon));

    btn
}

fn resolve_icon(icon_name: &str, theme_path: &str) -> gtk4::Image {
    if !icon_name.is_empty() && !Path::new(icon_name).is_absolute() && !theme_path.is_empty() {
        let candidates = [
            format!("{theme_path}/{icon_name}.png"),
            format!("{theme_path}/{icon_name}.svg"),
            format!("{theme_path}/16x16/apps/{icon_name}.png"),
            format!("{theme_path}/22x22/apps/{icon_name}.png"),
            format!("{theme_path}/apps/16/{icon_name}.png"),
            format!("{theme_path}/apps/22/{icon_name}.png"),
            format!("{theme_path}/apps/48/{icon_name}.png"),
        ];
        if let Some(path) = candidates.iter().find(|path| Path::new(path).is_file()) {
            let img = gtk4::Image::from_file(path);
            img.set_pixel_size(16);
            return img;
        }
    }

    get_theme_or_svg_icon(icon_name, 16)
}
