//! Calendar & Notifications dropdown window controller

pub mod render;

use chrono::Local;
use gtk4::prelude::*;
use infrastructure::i18n::trans;
use std::cell::RefCell;
use std::rc::Rc;
use crate::topbar::components::right::control_center::setup_click_outside_dismiss;

pub type WindowSlot = Rc<RefCell<Option<gtk4::ApplicationWindow>>>;

/// Formats the current local date header string according to the locale.
fn format_current_date() -> String {
    let now = Local::now();
    let weekday_key = format!("weekday.{}", now.format("%a").to_string().to_lowercase());
    let weekday = trans(&weekday_key);
    let month_key = format!("month.{}", now.format("%m"));
    let month = trans(&month_key);

    let format_template = trans("panel.date_format");
    if format_template.contains("{weekday}") {
        format_template
            .replace("{weekday}", &weekday)
            .replace("{day}", &now.format("%d").to_string())
            .replace("{month}", &month)
    } else {
        now.format("%A, %d %B").to_string()
    }
}

/// Sets up notifications list inside the drawer.
fn setup_notifications_list(notif_stack: &gtk4::Box, clear_btn: &gtk4::Button) {
    let stack_c = notif_stack.clone();

    let render_empty = move || {
        while let Some(child) = stack_c.first_child() {
            stack_c.remove(&child);
        }

        let empty_label = gtk4::Label::new(Some(&trans("panel.no_notifications")));
        empty_label.add_css_class("notif-empty-label");
        empty_label.set_halign(gtk4::Align::Center);
        empty_label.set_valign(gtk4::Align::Center);
        empty_label.set_vexpand(true);
        stack_c.append(&empty_label);
    };

    render_empty();

    let render_empty_c = render_empty;
    clear_btn.connect_clicked(move |_| {
        render_empty_c();
    });
}

/// Shows the calendar popup window with historical notifications drawer.
pub fn show_calendar_window(
    app: &gtk4::Application,
    cw_clone: WindowSlot,
) -> gtk4::ApplicationWindow {
    let (c_win, blur_host, _main_box, date_label, _calendar, clear_btn, notif_stack) =
        render::build_calendar(app);

    // Update initial date string
    date_label.set_text(&format_current_date());

    // Setup notifications list
    setup_notifications_list(&notif_stack, &clear_btn);

    // Setup outside click dismiss
    setup_click_outside_dismiss(&c_win, &blur_host);

    let cw_inner = cw_clone;
    c_win.connect_close_request(move |_| {
        if let Ok(mut borrow) = cw_inner.try_borrow_mut() {
            *borrow = None;
        }
        glib::Propagation::Proceed
    });

    c_win.present();
    c_win
}

