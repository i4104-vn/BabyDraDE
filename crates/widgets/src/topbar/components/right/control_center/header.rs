//! Control Center Header Row UI

use gtk4::prelude::*;
use infrastructure::i18n::trans;
use ui_kit::components::create_icon_button;
use ui_kit::icons::get_icon;
use ui_kit::theme::{is_dark_mode, set_dark_mode};

/// Launches settings application.
fn launch_settings_app() {
    std::thread::spawn(|| {
        if std::process::Command::new("babydra-settings")
            .spawn()
            .is_ok()
        {
            return;
        }
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                let local_settings = parent.join("babydra-settings");
                if local_settings.exists()
                    && std::process::Command::new(&local_settings).spawn().is_ok()
                {
                    return;
                }
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let user_bin = std::path::PathBuf::from(home).join(".local/bin/babydra-settings");
            let _ = std::process::Command::new(user_bin).spawn();
        }
    });
}

/// Creates a new header row for Control Center.
pub fn create_header_row() -> gtk4::Box {
    let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    header_box.set_hexpand(true);

    let title = gtk4::Label::new(Some(&trans("control.title")));
    title.add_css_class("control-center-title");
    title.set_xalign(0.0);
    title.set_hexpand(true);

    let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    btn_box.set_halign(gtk4::Align::End);

    let is_dark = is_dark_mode();
    let theme_icon_name = if is_dark { "dark-mode" } else { "brightness" };
    let theme_tooltip = trans("control.dark_mode").replace('\n', " ");

    // Theme toggle button
    let theme_btn = create_icon_button(
        theme_icon_name,
        16,
        &["circle-btn"],
        Some(&theme_tooltip),
        || {},
    );
    let theme_btn_click = theme_btn.clone();
    theme_btn.connect_clicked(move |_| {
        let new_mode = !is_dark_mode();
        set_dark_mode(new_mode);
        let name = if new_mode { "dark-mode" } else { "brightness" };
        let new_icon = get_icon(name, 16);
        theme_btn_click.set_child(Some(&new_icon));
    });

    let settings_btn = create_icon_button(
        "settings",
        16,
        &["circle-btn"],
        Some(&trans("control.settings")),
        || {
            launch_settings_app();
        },
    );

    btn_box.append(&theme_btn);
    btn_box.append(&settings_btn);

    header_box.append(&title);
    header_box.append(&btn_box);

    header_box
}

