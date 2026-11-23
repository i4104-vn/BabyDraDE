//! PAM authentication status UI handling.

use gtk4::prelude::*;
use std::rc::Rc;
use std::time::Duration;

use super::super::auth::verify_password;

pub(super) fn setup_auth_handler(
    on_unlock: Rc<dyn Fn()>,
    entry: &gtk4::PasswordEntry,
    status_label: &gtk4::Label,
    card_box: &gtk4::Box,
) {
    let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    let entry_clone = entry.clone();
    let status_label_clone = status_label.clone();
    let card_clone = card_box.clone();

    entry.connect_activate(move |_| {
        let password = entry_clone.text().to_string();
        entry_clone.set_text("");

        if verify_password(&username, &password) {
            on_unlock();
        } else {
            status_label_clone.set_text(&services::trans("lock.status_incorrect"));
            status_label_clone.add_css_class("error");
            card_clone.add_css_class("shake-error");

            let status_lbl = status_label_clone.clone();
            let card_box_ref = card_clone.clone();
            glib::timeout_add_local_once(Duration::from_millis(1600), move || {
                status_lbl.set_text(&services::trans("lock.status"));
                status_lbl.remove_css_class("error");
                card_box_ref.remove_css_class("shake-error");
            });
        }
    });
}
