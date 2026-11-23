//! Greeter widget modules and shared UI helpers.

pub(crate) mod login;
pub(crate) mod splash;
pub(crate) mod top_bar;

use gtk4::prelude::*;

/// Builds the user avatar, falling back to the BabyDra logo.
pub(crate) fn create_avatar_img(size: i32) -> gtk4::Widget {
    let avatar_path = services::config::get_babydra_dir().join("avatar.png");
    if avatar_path.exists() {
        let picture = gtk4::Picture::for_filename(&avatar_path);
        picture.set_size_request(size, size);
        picture.set_can_shrink(true);
        picture.add_css_class("avatar-img");
        return picture.upcast();
    }

    let logo = ui_kit::icons::get_logo_png(size);
    logo.add_css_class("avatar-img");
    logo.upcast()
}

/// Reads the last logged-in user from the BabyDraDE config directory.
pub(crate) fn get_last_user() -> Option<String> {
    let path = services::config::get_babydra_dir().join("last_user");
    std::fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Persists the last logged-in user.
pub(crate) fn save_last_user(user: &str) {
    let config_dir = services::config::get_babydra_dir();
    if std::fs::create_dir_all(&config_dir).is_ok() {
        let _ = std::fs::write(config_dir.join("last_user"), user);
    }
}
