//! Greeter login panel: user selection, password entry and submit button.

use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, DropDown, Image, Label, Orientation, PasswordEntry, Spinner,
};

use super::{create_avatar_img, get_last_user};

pub(crate) struct LoginWidget {
    pub(crate) container: GtkBox,
    pub(crate) login_panel: GtkBox,
    pub(crate) user_dropdown: DropDown,
    pub(crate) users: Vec<String>,
    pub(crate) pass_entry: PasswordEntry,
    pub(crate) login_btn: Button,
    pub(crate) btn_spinner: Spinner,
    pub(crate) error_box: GtkBox,
    pub(crate) error_label: Label,
}

/// Lists regular login users from `/etc/passwd`, excluding root and system accounts.
pub(crate) fn get_system_users() -> Vec<String> {
    let mut normal_users = Vec::new();
    let mut fallback_users = Vec::new();

    if let Ok(content) = std::fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            let parts: Vec<&str> = line.trim().split(':').collect();
            if parts.len() < 7 {
                continue;
            }
            let username = parts[0];
            let uid: u32 = parts[2].parse().unwrap_or(0);
            let shell = parts[6];
            if username == "root" || username == "greeter" {
                continue;
            }
            let is_nologin = shell.ends_with("nologin")
                || shell.ends_with("false")
                || shell.ends_with("git-shell");
            if uid >= 1000 && uid != 65534 && !is_nologin {
                normal_users.push(username.to_string());
            } else if !is_nologin {
                fallback_users.push(username.to_string());
            }
        }
    }

    let mut users = if !normal_users.is_empty() {
        normal_users
    } else if !fallback_users.is_empty() {
        fallback_users
    } else {
        vec![std::env::var("USER").unwrap_or_else(|_| "user".to_string())]
    };
    users.sort();
    users.dedup();
    users
}

pub(crate) fn build() -> LoginWidget {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.set_valign(Align::End);
    container.set_halign(Align::Center);
    container.set_margin_bottom(80);
    container.add_css_class("login-box");

    let login_panel = GtkBox::new(Orientation::Vertical, 12);
    login_panel.add_css_class("login-panel");
    login_panel.set_halign(Align::Center);
    login_panel.set_valign(Align::Center);

    let avatar_ring = GtkBox::new(Orientation::Vertical, 0);
    avatar_ring.add_css_class("avatar-ring");
    avatar_ring.set_halign(Align::Center);
    avatar_ring.set_valign(Align::Center);
    let avatar_inner = GtkBox::new(Orientation::Vertical, 0);
    avatar_inner.add_css_class("avatar-inner");
    avatar_inner.set_halign(Align::Center);
    avatar_inner.set_valign(Align::Center);
    avatar_inner.append(&create_avatar_img(110));
    avatar_ring.append(&avatar_inner);

    let username_label = Label::new(Some(&services::trans("greeter.user")));
    username_label.add_css_class("login-username-label");
    username_label.set_halign(Align::Center);

    let users = get_system_users();
    let str_refs: Vec<&str> = users.iter().map(String::as_str).collect();

    let user_capsule = GtkBox::new(Orientation::Horizontal, 8);
    user_capsule.add_css_class("input-capsule");
    let user_icon = Image::from_icon_name("avatar-default-symbolic");
    user_icon.set_pixel_size(18);
    user_icon.set_valign(Align::Center);
    user_icon.add_css_class("input-icon");
    let user_dropdown = DropDown::from_strings(&str_refs);
    user_dropdown.add_css_class("login-dropdown");
    user_dropdown.set_hexpand(true);
    user_dropdown.set_valign(Align::Center);
    user_dropdown.set_cursor_from_name(Some("pointer"));
    user_capsule.append(&user_icon);
    user_capsule.append(&user_dropdown);

    let pass_capsule = GtkBox::new(Orientation::Horizontal, 8);
    pass_capsule.add_css_class("input-capsule");
    let pass_icon = Image::from_icon_name("dialog-password-symbolic");
    pass_icon.set_pixel_size(18);
    pass_icon.set_valign(Align::Center);
    pass_icon.add_css_class("input-icon");
    let pass_entry = PasswordEntry::new();
    pass_entry.set_placeholder_text(Some(&services::trans("greeter.password")));
    pass_entry.add_css_class("login-input");
    pass_entry.set_hexpand(true);
    pass_entry.set_valign(Align::Center);

    let login_btn = Button::with_label("➔");
    login_btn.add_css_class("action-arrow-btn");
    login_btn.set_cursor_from_name(Some("pointer"));
    login_btn.set_valign(Align::Center);

    let btn_spinner = Spinner::new();
    btn_spinner.set_size_request(16, 16);
    btn_spinner.set_halign(Align::Center);
    btn_spinner.set_valign(Align::Center);

    pass_capsule.append(&pass_icon);
    pass_capsule.append(&pass_entry);
    pass_capsule.append(&login_btn);

    let error_box = GtkBox::new(Orientation::Horizontal, 8);
    error_box.add_css_class("error-badge");
    error_box.set_halign(Align::Center);
    error_box.set_visible(false);
    let error_icon = Image::from_icon_name("dialog-error-symbolic");
    error_icon.set_pixel_size(16);
    let error_label = Label::new(None);
    error_label.add_css_class("error-msg");
    error_box.append(&error_icon);
    error_box.append(&error_label);

    login_panel.append(&avatar_ring);
    login_panel.append(&username_label);
    login_panel.append(&user_capsule);
    login_panel.append(&pass_capsule);
    login_panel.append(&error_box);
    container.append(&login_panel);

    let default_user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    let target_user = match get_last_user() {
        Some(last) if users.contains(&last) => last,
        _ if users.contains(&default_user) => default_user,
        _ => users.first().cloned().unwrap_or(default_user),
    };
    let initial_index = users
        .iter()
        .position(|user| user == &target_user)
        .unwrap_or(0);
    user_dropdown.set_selected(initial_index as u32);
    username_label.set_text(&target_user);

    let label_for_selection = username_label.clone();
    let users_for_selection = users.clone();
    user_dropdown.connect_selected_notify(move |dropdown| {
        if let Some(name) = users_for_selection.get(dropdown.selected() as usize) {
            label_for_selection.set_text(name);
        }
    });

    LoginWidget {
        container,
        login_panel,
        user_dropdown,
        users,
        pass_entry,
        login_btn,
        btn_spinner,
        error_box,
        error_label,
    }
}
