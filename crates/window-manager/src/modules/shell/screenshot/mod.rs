//! BabyDraDE Screenshot — direct compositor capture with an annotation editor.

// screenshot-editor feature crate).
pub(crate) mod editor;
pub(crate) mod models;

use gtk4::prelude::*;
use gtk4::Application;

/// Requests a full screenshot from the next compositor frame.
pub(crate) fn trigger() {
    capture::trigger();
}

/// Opens the annotation editor for a captured image.
pub(crate) fn open_editor(image_path: String) {
    let app = Application::builder()
        .application_id("org.babydra.screenshot")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_activate(move |app| {
        ui_kit::init_theme();
        let window = editor::build_editor_ui(app, &image_path);
        window.present();
    });

    let _exit_code = app.run_with_args::<&str>(&[]);
}

pub(crate) mod capture;
