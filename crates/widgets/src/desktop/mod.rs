//! Desktop Shell Window (Wayland Layer::Background)

pub mod grid;
pub mod style;
pub mod wallpaper;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Overlay};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::rc::Rc;

pub fn build_desktop_window(app: &Application) -> (ApplicationWindow, Rc<dyn Fn(bool)>) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("BabyDraDE Desktop")
        .decorated(false)
        .build();

    // Layer Shell: Layer::Background, anchor to all edges
    window.init_layer_shell();
    window.set_layer(Layer::Background);
    window.set_namespace(Some("babydra-desktop"));
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_exclusive_zone(-1);

    window.add_css_class("desktop-window");

    let overlay = Overlay::new();
    overlay.set_hexpand(true);
    overlay.set_vexpand(true);

    let (wallpaper_widget, freeze_wallpaper) = wallpaper::create_wallpaper_w();
    let (desktop_grid, _rebuild) = grid::create_desktop_grid(&window);

    overlay.set_child(Some(&wallpaper_widget));
    overlay.add_overlay(&desktop_grid);

    window.set_child(Some(&overlay));

    (window, freeze_wallpaper)
}

pub fn run() {
    if std::env::var_os("GDK_BACKEND").is_none() {
        std::env::set_var("GDK_BACKEND", "wayland");
    }

    let app = Application::builder()
        .application_id("org.babydrade.desktop")
        .flags(gtk4::gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_startup(|_| {
        style::load_desktop_style();
    });

    app.connect_activate(|app| {
        let (window, _freeze) = build_desktop_window(app);
        window.present();
    });

    app.run_with_args::<&str>(&[]);
}
