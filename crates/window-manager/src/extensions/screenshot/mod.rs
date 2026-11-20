//! BabyDraDE Screenshot — direct compositor capture with an annotation editor.

pub mod capture;
pub mod editor;

use gtk4::prelude::*;
use gtk4::Application;

/// Requests a full screenshot from the next compositor frame.
pub fn trigger() {
    capture::trigger();
}

/// Opens the annotation editor for a captured image.
pub fn open_editor(image_path: String) {
    let app = Application::builder()
        .application_id("org.babydra.screenshot")
        .build();

    app.connect_activate(move |app| {
        ui_kit::init_theme();
        let window = editor::build_editor_ui(app, &image_path);
        window.present();
    });

    let _exit_code = app.run_with_args::<&str>(&[]);
}
