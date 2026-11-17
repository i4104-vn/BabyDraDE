pub mod center;
pub mod left;
pub mod right;

use center::DynamicIsland;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, CenterBox};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use left::LeftPill;
use right::create_right_section;

/// Launches the BabyDraDE TopBar GTK4 Layer-Shell widget
pub fn run() {
    // Ensure GDK connects via Wayland backend for Layer Shell support
    if std::env::var_os("GDK_BACKEND").is_none() {
        std::env::set_var("GDK_BACKEND", "wayland");
    }

    let app = Application::builder()
        .application_id("org.babydrade.topbar")
        .flags(gtk4::gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_startup(|_| {
        services::config::ensure_babydra_dirs();
        ui_kit::init_theme();
    });

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("BabyDraDE TopBar")
            .decorated(false)
            .default_height(36)
            .build();

        window.add_css_class("panel-window");

        // Layer Shell configuration for Wayland
        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_namespace(Some("babydra-topbar"));
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Top, 8);
        window.set_margin(Edge::Left, 1);
        window.set_margin(Edge::Right, 1);
        window.set_exclusive_zone(38);
        window.set_keyboard_mode(KeyboardMode::None);

        // Topbar Layout: Left Pill | Dynamic Island | Right Pills (Tray + System Monitor + Status/Clock)
        let center_box = CenterBox::builder()
            .css_classes(["panel-box"])
            .hexpand(true)
            .build();

        window.set_child(Some(&center_box));
        window.present();

        // Present the shell surface first; tray discovery and status polling
        // are constructed on the next main-loop turn.
        let center_for_idle = center_box.clone();
        let window_for_idle = window.clone();
        glib::idle_add_local_once(move || {
            let left_pill = LeftPill::new();
            center_for_idle.set_start_widget(Some(left_pill.widget()));
            let dynamic_island = DynamicIsland::new();
            center_for_idle.set_center_widget(Some(dynamic_island.widget()));
            let right_section = create_right_section(&window_for_idle);
            center_for_idle.set_end_widget(Some(&right_section));
        });
    });

    app.run_with_args::<&str>(&[]);
}
