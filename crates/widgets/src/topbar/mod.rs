pub mod components;

use components::{create_right_section, DynamicIsland, LeftPill};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, CenterBox};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

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

        // 1. Left Pill (Arch icon + | + Active App + 3 functions + more)
        let left_pill = LeftPill::new();
        center_box.set_start_widget(Some(left_pill.widget()));

        // 2. Center Pill (Dynamic Island)
        let dynamic_island = DynamicIsland::new();
        center_box.set_center_widget(Some(dynamic_island.widget()));

        // 3. Right Section (Tray + System Monitor + Status + Clock)
        let right_section = create_right_section(&window);
        center_box.set_end_widget(Some(&right_section));

        window.set_child(Some(&center_box));
        window.present();
    });

    app.run_with_args::<&str>(&[]);
}

