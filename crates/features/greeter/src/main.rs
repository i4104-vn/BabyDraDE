//! BabyDraDE Greeter — greetd login screen.
//!
//! Launched directly by greetd; the compositor never calls it.

pub(crate) mod auth;
pub(crate) mod handler;
pub(crate) mod orientation;
pub(crate) mod render;
pub(crate) mod widgets;

use gtk4::gio::prelude::ApplicationExt;
use gtk4::prelude::*;

fn main() {
    if std::env::var_os("GREETD_SOCK").is_none() {
        tracing::warn!("GREETD_SOCK is not set; greeter login will fail until started by greetd");
    }

    let app = gtk4::Application::new(Some("org.babydrade.greeter"), Default::default());
    app.connect_activate(|app| {
        orientation::apply();
        let greeter = render::build_greeter_ui(app);
        handler::setup_handlers(&greeter);
        greeter.window.present();
    });
    let _exit_code = app.run_with_args::<&str>(&[]);
}
