use gtk4::prelude::*;
use services::tray::TrayItem;
use std::path::Path;
use ui_kit::icons::{get_icon, resolve_icon_pair};

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
    btn.set_child(Some(&resolve_icon(&item.icon_name, &item.icon_theme_path)));

    btn
}

/// Resolves a Status Notifier icon using the same path as the other topbar
/// items: BabyDra's own theme-aware SVG set first, system icon theme only for
/// names that are not part of it.
fn resolve_icon(icon_name: &str, theme_path: &str) -> gtk4::Image {
    if icon_name.is_empty() {
        return get_icon("image-missing", TRAY_ICON_SIZE);
    }

    // SNI items may advertise an absolute path or a private icon theme folder.
    if Path::new(icon_name).is_absolute() && Path::new(icon_name).is_file() {
        return image_from_file(icon_name);
    }
    if !theme_path.is_empty() {
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
            return image_from_file(path);
        }
    }

    // Same helper used by the rest of the panel: local SVG (with aliases and
    // dark/light variants) and automatic refresh when the theme switches.
    if resolve_icon_pair(icon_name).is_some() {
        return get_icon(icon_name, TRAY_ICON_SIZE);
    }

    // Arbitrary freedesktop names advertised by apps (e.g. "spotify") only
    // exist in the system theme.
    if let Some(display) = gtk4::gdk::Display::default() {
        if gtk4::IconTheme::for_display(&display).has_icon(icon_name) {
            let img = gtk4::Image::from_icon_name(icon_name);
            img.set_pixel_size(TRAY_ICON_SIZE);
            return img;
        }
    }

    get_icon("image-missing", TRAY_ICON_SIZE)
}

fn image_from_file(path: impl AsRef<Path>) -> gtk4::Image {
    let img = gtk4::Image::from_file(path.as_ref());
    img.set_pixel_size(TRAY_ICON_SIZE);
    img
}
