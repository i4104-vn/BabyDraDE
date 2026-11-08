mod components;
mod style;

use components::{DynamicIsland, LeftPill, StatusPill, TrayPill};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, CenterBox, Orientation};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use style::load_topbar_style;

fn main() {
    let app = Application::builder()
        .application_id("org.babydrade.topbar")
        .build();

    app.connect_startup(|_| {
        load_topbar_style();
    });

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("BabyDraDE TopBar")
            .decorated(false)
            .build();

        // Layer Shell configuration for Wayland
        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_namespace(Some("babydra-topbar"));
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Top, 4);
        window.set_margin(Edge::Left, 8);
        window.set_margin(Edge::Right, 8);
        window.set_exclusive_zone(38);

        // Topbar Layout: Left Pill | Dynamic Island | Right Pills (Tray + Status)
        let center_box = CenterBox::builder()
            .css_classes(["topbar-box"])
            .hexpand(true)
            .build();

        // 1. Left Pill (Arch icon + | + Active App + 3 functions + more)
        let left_pill = LeftPill::new();
        center_box.set_start_widget(Some(left_pill.widget()));

        // 2. Center Pill (Dynamic Island)
        let dynamic_island = DynamicIsland::new();
        center_box.set_center_widget(Some(dynamic_island.widget()));

        // 3. Right Section (2 separate pills: Tray Pill + Status Pill)
        let right_container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let tray_pill = TrayPill::new();
        let status_pill = StatusPill::new();

        right_container.append(tray_pill.widget());
        right_container.append(status_pill.widget());

        center_box.set_end_widget(Some(&right_container));

        window.set_child(Some(&center_box));
        window.present();
    });

    app.run();
}
