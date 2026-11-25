//! Wallpaper, avatar and transition screenshot image handling.

use gtk4::prelude::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn build_wallpaper_img(custom_path: Option<&str>) -> gtk4::Picture {
    let bg_picture = gtk4::Picture::new();
    bg_picture.set_can_shrink(true);
    bg_picture.set_hexpand(true);
    bg_picture.set_vexpand(true);

    if let Some(path) = custom_path {
        bg_picture.set_filename(Some(path));
        return bg_picture;
    }

    if let Some(path) = services::wallpaper::get_wallpaper() {
        bg_picture.set_filename(Some(&path));
        return bg_picture;
    }

    bg_picture
}

pub(crate) fn build_avatar_widget(size: i32) -> gtk4::Widget {
    let avatar_path = services::config::get_babydra_dir().join("avatar.png");
    if avatar_path.exists() {
        let pic = gtk4::Picture::for_filename(&avatar_path);
        pic.set_size_request(size, size);
        pic.set_can_shrink(true);
        pic.add_css_class("lock-avatar");
        return pic.upcast();
    }

    let avatar_icon = ui_kit::icons::get_icon("avatar-default", size);
    avatar_icon.add_css_class("lock-avatar");
    avatar_icon.set_halign(gtk4::Align::Center);
    avatar_icon.set_valign(gtk4::Align::Center);
    avatar_icon.upcast()
}

pub(crate) fn capture_transition_images() -> Vec<Option<PathBuf>> {
    let Some(display) = gtk4::gdk::Display::default() else {
        return Vec::new();
    };

    let monitors = display.monitors();
    let count = monitors.n_items();
    if count == 0 {
        return vec![capture_transition_image(None, 0)];
    }

    (0..count)
        .map(|index| {
            let monitor = monitors
                .item(index)
                .and_then(|obj| obj.downcast::<gtk4::gdk::Monitor>().ok());
            monitor
                .as_ref()
                .and_then(|monitor| capture_transition_image(monitor.connector().as_deref(), index))
        })
        .collect()
}

fn capture_transition_image(output: Option<&str>, index: u32) -> Option<PathBuf> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let prefix = format!("babydra-lock-{}-{index}-{nonce}", std::process::id());
    let raw_path = std::env::temp_dir().join(format!("{prefix}-raw.png"));

    let mut grim = std::process::Command::new("grim");
    if let Some(output) = output {
        grim.args(["-o", output]);
    }
    let captured = grim
        .args(["-t", "png", "-l", "1"])
        .arg(&raw_path)
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    if !captured {
        let _ = std::fs::remove_file(raw_path);
        return None;
    }

    Some(raw_path)
}
