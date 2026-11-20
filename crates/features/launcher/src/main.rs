//! BabyDraDE Launcher executable — quick app search overlay.

use babydra_launcher::build_launcher_ui;
use gtk4::prelude::*;
use gtk4::Application;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();

    if std::env::var_os("GDK_BACKEND").is_none() {
        std::env::set_var("GDK_BACKEND", "wayland");
    }

    let app = Application::builder()
        .application_id("org.babydra.launcher")
        .flags(gtk4::gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    let window_slot: Rc<RefCell<Option<gtk4::ApplicationWindow>>> = Rc::new(RefCell::new(None));

    let activate_slot = window_slot.clone();
    app.connect_activate(move |app| {
        ui_kit::init_theme();
        if let Some(window) = activate_slot.borrow().as_ref() {
            if window.is_visible() {
                window.set_visible(false);
            } else {
                window.present();
            }
            return;
        }
        let window = build_launcher_ui(app, activate_slot.clone());
        window.present();
        *activate_slot.borrow_mut() = Some(window);
    });

    let hide_slot = window_slot.clone();
    app.connect_command_line(move |app, command_line| {
        let hide = command_line
            .arguments()
            .iter()
            .any(|arg| arg.to_string_lossy() == "--hide");
        if hide {
            if let Some(window) = hide_slot.borrow().as_ref() {
                window.set_visible(false);
            }
        } else {
            app.activate();
        }
        gtk4::glib::ExitCode::SUCCESS
    });

    let exit_code = app.run();
    std::process::exit(exit_code.get() as i32);
}
