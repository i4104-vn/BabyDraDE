//! Signal wiring for the greeter UI: clock, splash, power actions and login.

use gtk4::prelude::*;
use std::sync::mpsc;
use std::time::Duration;

use super::auth;
use super::render::GreeterWidgets;
use super::widgets::{save_last_user, top_bar::TopBarWidget};

pub(crate) fn setup_handlers(greeter: &GreeterWidgets) {
    setup_clock(&greeter.top_bar);
    setup_splash_transition(greeter);
    setup_power_buttons(&greeter.top_bar);
    setup_login_flow(greeter);
}

fn format_date(fmt: &str, now: &chrono::DateTime<chrono::Local>) -> String {
    fmt.replace("{weekday}", &now.format("%A").to_string())
        .replace("{day}", &now.format("%d").to_string())
        .replace("{month}", &now.format("%B").to_string())
        .replace("{year}", &now.format("%Y").to_string())
}

fn setup_clock(top_bar: &TopBarWidget) {
    let date_format = services::trans("greeter.date_format");
    let update = |clock: &gtk4::Label, date: &gtk4::Label| {
        let now = chrono::Local::now();
        clock.set_text(&now.format("%H:%M").to_string());
        date.set_text(&format_date(&date_format, &now));
    };
    update(&top_bar.clock_label, &top_bar.date_label);

    let clock_label = top_bar.clock_label.clone();
    let date_label = top_bar.date_label.clone();
    glib::timeout_add_seconds_local(1, move || {
        let now = chrono::Local::now();
        clock_label.set_text(&now.format("%H:%M").to_string());
        date_label.set_text(&format_date(&date_format, &now));
        glib::ControlFlow::Continue
    });
}

fn setup_splash_transition(greeter: &GreeterWidgets) {
    greeter.login.container.set_opacity(0.0);
    greeter.login.container.set_visible(false);
    greeter.splash.container.set_visible(true);
    greeter.splash.container.set_opacity(1.0);

    let splash = greeter.splash.container.clone();
    let login = greeter.login.container.clone();
    let pass_entry = greeter.login.pass_entry.clone();
    glib::timeout_add_seconds_local(2, move || {
        splash.set_opacity(0.0);
        splash.set_visible(false);
        login.set_visible(true);
        login.set_opacity(1.0);
        pass_entry.grab_focus();
        glib::ControlFlow::Break
    });
}

fn power_action(action: &str) {
    if let Err(err) = std::process::Command::new("systemctl").arg(action).spawn() {
        tracing::warn!("greeter power action '{action}' failed: {err}");
    }
}

fn setup_power_buttons(top_bar: &TopBarWidget) {
    top_bar
        .power_btn
        .connect_clicked(|_| power_action("poweroff"));
    top_bar
        .reboot_btn
        .connect_clicked(|_| power_action("reboot"));
    top_bar
        .suspend_btn
        .connect_clicked(|_| power_action("suspend"));
}

#[derive(Clone)]
struct LoginRefs {
    user_dropdown: gtk4::DropDown,
    users: Vec<String>,
    pass_entry: gtk4::PasswordEntry,
    login_btn: gtk4::Button,
    btn_spinner: gtk4::Spinner,
    power_btn: gtk4::Button,
    reboot_btn: gtk4::Button,
    suspend_btn: gtk4::Button,
    error_label: gtk4::Label,
    error_box: gtk4::Box,
    login_panel: gtk4::Box,
}

impl LoginRefs {
    fn from_greeter(greeter: &GreeterWidgets) -> Self {
        Self {
            user_dropdown: greeter.login.user_dropdown.clone(),
            users: greeter.login.users.clone(),
            pass_entry: greeter.login.pass_entry.clone(),
            login_btn: greeter.login.login_btn.clone(),
            btn_spinner: greeter.login.btn_spinner.clone(),
            power_btn: greeter.top_bar.power_btn.clone(),
            reboot_btn: greeter.top_bar.reboot_btn.clone(),
            suspend_btn: greeter.top_bar.suspend_btn.clone(),
            error_label: greeter.login.error_label.clone(),
            error_box: greeter.login.error_box.clone(),
            login_panel: greeter.login.login_panel.clone(),
        }
    }

    fn set_controls_sensitive(&self, sensitive: bool) {
        self.user_dropdown.set_sensitive(sensitive);
        self.pass_entry.set_sensitive(sensitive);
        self.login_btn.set_sensitive(sensitive);
        self.power_btn.set_sensitive(sensitive);
        self.reboot_btn.set_sensitive(sensitive);
        self.suspend_btn.set_sensitive(sensitive);
    }
}

fn setup_login_flow(greeter: &GreeterWidgets) {
    let refs = LoginRefs::from_greeter(greeter);

    let on_submit = {
        let refs = refs.clone();
        move || {
            if !refs.login_btn.is_sensitive() {
                return;
            }

            let user = refs
                .users
                .get(refs.user_dropdown.selected() as usize)
                .cloned()
                .unwrap_or_default();
            let password = refs.pass_entry.text().to_string();
            if user.is_empty() || password.is_empty() {
                return;
            }

            save_last_user(&user);
            refs.set_controls_sensitive(false);
            refs.btn_spinner.start();
            refs.login_btn.set_child(Some(&refs.btn_spinner));

            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || {
                let _ = tx.send(auth::do_login(user, password));
            });

            let completed = refs.clone();
            glib::timeout_add_local(Duration::from_millis(50), move || match rx.try_recv() {
                Ok(Ok(())) => {
                    completed.error_box.set_visible(false);
                    glib::ControlFlow::Break
                }
                Ok(Err(err)) => {
                    completed.btn_spinner.stop();
                    completed.login_btn.set_child(Option::<&gtk4::Widget>::None);
                    completed.login_btn.set_label("➔");
                    completed.set_controls_sensitive(true);
                    completed.error_label.set_text(&err);
                    completed.error_box.set_visible(true);
                    completed.pass_entry.set_text("");
                    completed.pass_entry.grab_focus();
                    completed.login_panel.add_css_class("shake-error");
                    let panel = completed.login_panel.clone();
                    glib::timeout_add_local_once(Duration::from_millis(400), move || {
                        panel.remove_css_class("shake-error");
                    });
                    glib::ControlFlow::Break
                }
                Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
            });
        }
    };

    let submit_button = on_submit.clone();
    greeter
        .login
        .login_btn
        .connect_clicked(move |_| submit_button());
    greeter
        .login
        .pass_entry
        .connect_activate(move |_| on_submit());
}
