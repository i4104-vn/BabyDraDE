//! Greeter window construction and layout assembly.

use gtk4::prelude::*;
use gtk4::{Align, ApplicationWindow, Box as GtkBox, Orientation, Overlay, Picture};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use super::widgets::{login, splash, top_bar};
use super::widgets::{login::LoginWidget, splash::SplashWidget, top_bar::TopBarWidget};

pub(crate) struct GreeterWidgets {
    pub(crate) window: ApplicationWindow,
    pub(crate) top_bar: TopBarWidget,
    pub(crate) splash: SplashWidget,
    pub(crate) login: LoginWidget,
}

fn load_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(include_str!("../resources/greeter.css"));
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_USER,
        );
    }
}

fn build_wallpaper() -> Picture {
    let picture = Picture::new();
    picture.set_can_shrink(true);
    picture.set_hexpand(true);
    picture.set_vexpand(true);
    if let Some(path) = services::wallpaper::get_wallpaper() {
        picture.set_filename(Some(&path));
    }
    picture
}

fn init_layer_window(window: &ApplicationWindow) {
    if !gtk4_layer_shell::is_supported() {
        return;
    }
    window.init_layer_shell();
    window.set_namespace(Some("babydra-greeter"));
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.set_exclusive_zone(-1);
    for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
        window.set_anchor(edge, true);
    }
}

pub(crate) fn build_greeter_ui(app: &gtk4::Application) -> GreeterWidgets {
    ui_kit::init_theme();

    let window = ApplicationWindow::builder()
        .application(app)
        .title(services::trans("greeter.os_name"))
        .decorated(false)
        .build();

    init_layer_window(&window);
    window.fullscreen();
    load_css();

    let overlay = Overlay::new();
    overlay.set_child(Some(&build_wallpaper()));

    let tint = GtkBox::new(Orientation::Vertical, 0);
    tint.add_css_class("greeter-tint");
    overlay.add_overlay(&tint);

    let top_bar_widget = top_bar::build();
    let splash_widget = splash::build();
    let login_widget = login::build();
    overlay.add_overlay(&top_bar_widget.container);

    let center = GtkBox::new(Orientation::Vertical, 0);
    center.set_valign(Align::Center);
    center.set_halign(Align::Center);
    center.append(&splash_widget.container);
    center.append(&login_widget.container);
    overlay.add_overlay(&center);

    window.set_child(Some(&overlay));

    GreeterWidgets {
        window,
        top_bar: top_bar_widget,
        splash: splash_widget,
        login: login_widget,
    }
}
