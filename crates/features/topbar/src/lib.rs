pub mod center;
pub mod left;
pub mod right;

use center::DynamicIsland;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, CenterBox};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use left::LeftPill;
use right::create_right_section;
use std::cell::RefCell;

thread_local! {
    static DISMISS_LAYER: RefCell<Option<ApplicationWindow>> = const { RefCell::new(None) };
}

/// Panel offsets from the monitor edges, shared with SNI menu placement.
pub(crate) const PANEL_MARGIN_X: i32 = 1;
pub(crate) const PANEL_MARGIN_Y: i32 = 8;

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
        services::tray::start_tray_host();
    });

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("BabyDraDE TopBar")
            .decorated(false)
            .default_height(36)
            .build();

        window.add_css_class("panel-window");

        // Clicking anywhere on the panel outside the island capsule dismisses
        // any open island popover (GTK autohide is unreliable on layer shell).
        let dismiss_window = window.clone();
        let outside_click = gtk4::GestureClick::new();
        outside_click.connect_pressed(move |_, _, x, y| {
            center::ui::popover::dismiss_outside(dismiss_window.upcast_ref(), x, y);
            ui_kit::overlay::context_menu::dismiss_all();
        });
        window.add_controller(outside_click);

        // Layer Shell configuration for Wayland
        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_namespace(Some("babydra-topbar"));
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Top, PANEL_MARGIN_Y);
        window.set_margin(Edge::Left, PANEL_MARGIN_X);
        window.set_margin(Edge::Right, PANEL_MARGIN_X);
        window.set_exclusive_zone(38);
        window.set_keyboard_mode(KeyboardMode::None);

        // Topbar Layout: Left Pill | Dynamic Island | Right Pills (Tray + System Monitor + Status/Clock)
        let center_box = CenterBox::builder()
            .css_classes(["panel-box"])
            .hexpand(true)
            .build();

        let left_pill = LeftPill::new();
        center_box.set_start_widget(Some(left_pill.widget()));
        let dynamic_island = DynamicIsland::new(app);
        center_box.set_center_widget(Some(dynamic_island.widget()));
        let right_section = create_right_section(&window);
        center_box.set_end_widget(Some(&right_section));

        window.set_child(Some(&center_box));
        window.present();
        DISMISS_LAYER.with(|slot| *slot.borrow_mut() = Some(create_dismiss_layer(app)));
        ui_kit::overlay::context_menu::set_visibility_hook(|open| {
            DISMISS_LAYER.with(|slot| {
                if let Some(window) = slot.borrow().as_ref() {
                    window.set_visible(open);
                }
            });
        });
    });

    app.run_with_args::<&str>(&[]);
}

fn create_dismiss_layer(app: &Application) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("BabyDraDE Dismiss Layer")
        .decorated(false)
        .build();
    window.add_css_class("dismiss-layer");

    window.init_layer_shell();
    // Bottom keeps it above the desktop but below normal application windows.
    window.set_layer(Layer::Bottom);
    window.set_namespace(Some("babydra-dismiss"));
    for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
        window.set_anchor(edge, true);
        window.set_margin(edge, 0);
    }
    window.set_exclusive_zone(0);
    window.set_keyboard_mode(KeyboardMode::None);

    let gesture = gtk4::GestureClick::new();
    let catcher = window.clone();
    gesture.connect_pressed(move |_, _, _, _| {
        catcher.set_visible(false);
        ui_kit::overlay::context_menu::dismiss_all();
        center::ui::popover::dismiss_all_popovers();
    });
    window.add_controller(gesture);
    window.set_visible(false);
    window
}
