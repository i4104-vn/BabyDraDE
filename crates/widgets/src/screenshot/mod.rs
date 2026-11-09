//! BabyDraDE Screenshot — Regional and fullscreen screen capture with annotation editor.

pub mod capture;
pub mod editor;
pub mod model;
pub mod render;

use capture::{capture_fullscreen, capture_screen};
use editor::build_editor_ui;
use gtk4::prelude::*;
use gtk4::Application;

/// Runs the screenshot widget.
pub fn run() {
    let args: Vec<String> = std::env::args().collect();

    // 1. Handle Fullscreen Immediate Capture
    if args.iter().any(|arg| arg == "--full") {
        capture_fullscreen();
        return;
    }

    // 2. Interactive Regional Capture (Default)
    let temp_path = match capture_screen() {
        Some(path) => path,
        None => return,
    };

    let temp_path_for_activate = temp_path.clone();
    let temp_path_for_cleanup = temp_path.clone();

    let app = Application::builder()
        .application_id("org.babydra.screenshot")
        .build();

    app.connect_activate(move |app| {
        ui_kit::init_theme();
        let window = build_editor_ui(app, &temp_path_for_activate);
        window.present();
    });

    let _exit_code = app.run_with_args::<&str>(&[]);

    // Clean up temporary screenshot file on exit
    let _ = std::fs::remove_file(&temp_path_for_cleanup);
}
