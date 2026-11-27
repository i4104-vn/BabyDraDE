//! Calendar & Notifications dropdown window controller

pub mod render {
    //! UI construction for the dropdown calendar window.

    use gtk4::prelude::*;
    use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
    use services::i18n::trans;

    /// Builds the calendar popup window anchored to the top-right.
    pub fn build_calendar(
        app: &gtk4::Application,
    ) -> (
        gtk4::ApplicationWindow,
        gtk4::Box,
        gtk4::Box,
        gtk4::Label,
        gtk4::Calendar,
        gtk4::Button,
        gtk4::Box,
    ) {
        let c_win = gtk4::ApplicationWindow::builder()
            .application(app)
            .title("BabyDraDE Calendar")
            .decorated(false)
            .build();

        c_win.init_layer_shell();
        c_win.set_layer(Layer::Overlay);
        c_win.set_namespace(Some("babydra-calendar"));
        c_win.set_anchor(Edge::Top, true);
        c_win.set_anchor(Edge::Bottom, true);
        c_win.set_anchor(Edge::Left, true);
        c_win.set_anchor(Edge::Right, true);
        c_win.set_keyboard_mode(KeyboardMode::OnDemand);

        c_win.add_css_class("calendar-window");

        let blur_host = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        blur_host.add_css_class("calendar-blur-host");
        blur_host.set_halign(gtk4::Align::End);
        blur_host.set_valign(gtk4::Align::Start);
        blur_host.set_size_request(360, 520);
        blur_host.set_margin_top(6);
        blur_host.set_margin_end(12);

        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        main_box.add_css_class("calendar-box");
        main_box.set_vexpand(true);
        main_box.set_hexpand(true);

        // 1. Header: Date on left
        let top_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        top_row.add_css_class("calendar-top-row");

        let date_label = gtk4::Label::new(None);
        date_label.add_css_class("calendar-top-date");
        date_label.set_halign(gtk4::Align::Start);
        date_label.set_hexpand(true);

        top_row.append(&date_label);
        main_box.append(&top_row);

        // 2. Calendar widget
        let calendar = gtk4::Calendar::new();
        calendar.add_css_class("calendar-widget");
        main_box.append(&calendar);

        // 3. Notifications Header
        let notif_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        notif_header.set_hexpand(true);

        let notif_title = gtk4::Label::new(Some(&trans("panel.notifications")));
        notif_title.add_css_class("notif-panel-title");
        notif_title.set_halign(gtk4::Align::Start);
        notif_title.set_hexpand(true);

        let clear_btn = gtk4::Button::with_label(&trans("panel.clear_all"));
        clear_btn.add_css_class("clear-all-btn");
        clear_btn.set_halign(gtk4::Align::End);

        notif_header.append(&notif_title);
        notif_header.append(&clear_btn);
        main_box.append(&notif_header);

        // 4. Notifications Scrolled Window & List Box
        let scrolled_win = gtk4::ScrolledWindow::new();
        scrolled_win.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        scrolled_win.set_vexpand(true);
        scrolled_win.set_hexpand(true);
        scrolled_win.add_css_class("notif-scroll");

        let notif_stack = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        notif_stack.add_css_class("notif-stack-box");
        notif_stack.set_vexpand(true);
        notif_stack.set_valign(gtk4::Align::Start);

        scrolled_win.set_child(Some(&notif_stack));
        main_box.append(&scrolled_win);

        blur_host.append(&main_box);
        c_win.set_child(Some(&blur_host));

        (
            c_win,
            blur_host,
            main_box,
            date_label,
            calendar,
            clear_btn,
            notif_stack,
        )
    }
}

use crate::control::control_center::{
    register_panel, setup_click_outside_dismiss, unregister_panel,
};
use chrono::Local;
use gtk4::prelude::*;
use services::i18n::trans;
use std::cell::RefCell;
use std::rc::Rc;

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
    let c_win_weak_close = c_win.downgrade();
    c_win.connect_close_request(move |_| {
        if let Some(win) = c_win_weak_close.upgrade() {
            unregister_panel(&win);
        }
        if let Ok(mut borrow) = cw_inner.try_borrow_mut() {
            *borrow = None;
        }
        glib::Propagation::Proceed
    });

    register_panel(&c_win);
    c_win.present();
    c_win
}
