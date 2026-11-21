//! Clipboard history picker (Super+V).
//!
//! Runs in its own subprocess because the compositor process already owns the
//! default GLib main context, which GTK cannot share.

use gtk4::prelude::*;
use tracing::warn;

/// Launches the clipboard picker as a separate process.
pub fn trigger_picker() {
    let exe = match std::env::current_exe() {
        Ok(path) => path,
        Err(err) => {
            warn!("Cannot locate compositor executable for clipboard picker: {err}");
            return;
        }
    };

    let mut command = std::process::Command::new(exe);
    command.arg("--clipboard-picker");
    if let Ok(socket) = std::env::var("WAYLAND_DISPLAY") {
        command.env("WAYLAND_DISPLAY", socket);
    }
    if let Err(err) = command.spawn() {
        warn!("Failed to open clipboard picker: {err}");
    }
}

/// Copies raw bytes into the Wayland selection using `wl-copy` (wl-clipboard).
fn copy_to_selection(data: &[u8], mime: &str) {
    use std::io::Write;
    use std::process::Stdio;

    // Preserve the original target type (e.g. image/png) so apps paste the
    // image itself instead of treating the bytes as text.
    let mut child = match std::process::Command::new("wl-copy")
        .args(["-t", mime])
        .stdin(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            warn!("wl-copy is unavailable; install wl-clipboard: {err}");
            return;
        }
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(data);
    }
    let _ = child.wait();
}

/// Builds and runs the GTK clipboard history window.
pub fn open_picker() {
    let app = gtk4::Application::builder()
        .application_id("org.babydra.clipboard")
        .build();

    app.connect_activate(|app| {
        ui_kit::init_theme();

        let window = gtk4::ApplicationWindow::new(app);
        window.set_title(Some("Clipboard History"));
        window.set_default_size(540, 440);

        let main_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(8)
            .build();

        let scroll = gtk4::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();
        let list = gtk4::ListBox::new();
        scroll.set_child(Some(&list));
        main_box.append(&scroll);

        let entries = services::system::clipboard::history();
        if entries.is_empty() {
            let empty = gtk4::Label::new(Some("Nothing copied yet"));
            empty.set_margin_top(24);
            empty.set_margin_bottom(24);
            list.append(&empty);
        }

        for entry in entries {
            let label = if entry.preview.chars().count() > 110 {
                let short: String = entry.preview.chars().take(110).collect();
                format!("{short}…")
            } else {
                entry.preview.clone()
            };
            let button = gtk4::Button::with_label(&label);
            let window_weak = window.downgrade();
            button.connect_clicked(move |_| {
                if !entry.data.is_empty() {
                    copy_to_selection(&entry.data, &entry.mime);
                }
                if let Some(window) = window_weak.upgrade() {
                    window.destroy();
                }
            });
            let row = gtk4::ListBoxRow::new();
            row.set_child(Some(&button));
            list.append(&row);
        }

        let clear = gtk4::Button::with_label("Clear history");
        let window_weak = window.downgrade();
        clear.connect_clicked(move |_| {
            services::system::clipboard::clear_history();
            if let Some(window) = window_weak.upgrade() {
                window.destroy();
            }
        });
        main_box.append(&clear);

        window.set_child(Some(&main_box));
        window.present();
    });

    let _exit_code = app.run_with_args::<&str>(&[]);
}
