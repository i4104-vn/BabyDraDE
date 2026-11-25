//! Lock screen widget layout and clock construction.

use gtk4::prelude::*;
use std::time::Duration;

use super::wallpaper::build_avatar_widget;

pub(crate) use crate::modules::shell::lock::models::PrimaryCard;

pub(crate) fn build_clock_labels() -> (gtk4::Label, gtk4::Label) {
    let clock_label = gtk4::Label::new(None);
    clock_label.add_css_class("lock-clock");

    let date_label = gtk4::Label::new(None);
    date_label.add_css_class("lock-date");

    (clock_label, date_label)
}

pub(crate) fn build_primary_card(
    clock_label: &gtk4::Label,
    date_label: &gtk4::Label,
) -> PrimaryCard {
    let card_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    card_box.add_css_class("lock-card");
    card_box.set_valign(gtk4::Align::Center);
    card_box.set_halign(gtk4::Align::Center);

    let avatar_widget = build_avatar_widget(110);

    let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    let user_label = gtk4::Label::new(Some(&username));
    user_label.add_css_class("lock-username");

    let entry = gtk4::PasswordEntry::new();
    entry.set_placeholder_text(Some(&services::trans("lock.placeholder")));
    entry.add_css_class("lock-input");
    entry.set_halign(gtk4::Align::Center);

    let status_label = gtk4::Label::new(Some(&services::trans("lock.status")));
    status_label.add_css_class("lock-status");

    card_box.append(clock_label);
    card_box.append(date_label);
    card_box.append(&avatar_widget);
    card_box.append(&user_label);
    card_box.append(&entry);
    card_box.append(&status_label);

    PrimaryCard {
        card_box,
        entry,
        status_label,
    }
}

pub(super) fn setup_clock_timer(clock_label: &gtk4::Label, date_label: &gtk4::Label) {
    let update = |cl: &gtk4::Label, dl: &gtk4::Label| {
        let now = chrono::Local::now();
        let time = now.format("%H:%M").to_string();
        let date = now.format("%A, %B %d").to_string();
        cl.set_text(&time);
        dl.set_text(&date);
    };
    update(clock_label, date_label);

    let cl = clock_label.clone();
    let dl = date_label.clone();
    glib::timeout_add_seconds_local(1, move || {
        let now = chrono::Local::now();
        let time = now.format("%H:%M").to_string();
        let date = now.format("%A, %B %d").to_string();
        cl.set_text(&time);
        dl.set_text(&date);
        glib::ControlFlow::Continue
    });
}

pub(super) fn setup_focus_management(
    window: &gtk4::ApplicationWindow,
    entry: &gtk4::PasswordEntry,
) {
    let entry_focus = entry.clone();
    glib::timeout_add_local_once(Duration::from_millis(100), move || {
        entry_focus.grab_focus();
    });

    let entry_click = entry.clone();
    let click_gesture = gtk4::GestureClick::new();
    click_gesture.connect_pressed(move |_, _, _, _| {
        entry_click.grab_focus();
    });
    window.add_controller(click_gesture);
}
