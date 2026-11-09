//! Bluetooth Tile UI for Control Center

use gtk4::glib;
use infrastructure::i18n::trans;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use ui_kit::components::{create_toggle_tile, update_toggle_state};

/// Checks if bluetooth is powered on via bluetoothctl
fn is_bluetooth_enabled() -> bool {
    if let Ok(output) = Command::new("bluetoothctl").arg("show").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.contains("Powered: yes")
    } else {
        false
    }
}

/// Toggles bluetooth power via bluetoothctl
fn set_bluetooth_enabled(enable: bool) {
    let arg = if enable { "on" } else { "off" };
    let _ = Command::new("bluetoothctl").args(["power", arg]).output();
}

/// Creates a new Bluetooth toggle tile.
pub fn create_bt_tile() -> gtk4::Button {
    let title = trans("control.bluetooth");
    let initial_on = Arc::new(AtomicBool::new(false));

    let initial_on_clone = initial_on.clone();
    let (btn, sub_label) = create_toggle_tile(
        "bluetooth",
        &title,
        &trans("control.off"),
        "",
        false,
        move |new_active| {
            initial_on_clone.store(new_active, Ordering::SeqCst);
            std::thread::spawn(move || {
                set_bluetooth_enabled(new_active);
            });
        },
    );

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(bool, String)>();
    let sub_lbl_c = sub_label;
    let btn_c = btn.clone();
    glib::spawn_future_local(async move {
        if let Some((on, sub_text)) = rx.recv().await {
            sub_lbl_c.set_text(&sub_text);
            update_toggle_state(&btn_c, on, "bluetooth");
        }
    });

    // Query initial state in background thread and notify main thread
    std::thread::spawn(move || {
        let on = is_bluetooth_enabled();
        initial_on.store(on, Ordering::SeqCst);
        let sub_text = if on {
            trans("control.on")
        } else {
            trans("control.off")
        };
        let _ = tx.send((on, sub_text));
    });

    btn
}
