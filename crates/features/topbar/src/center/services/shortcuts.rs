//! Hardcoded island shortcuts — no configuration file involved.
//!
//! | Keys | Trigger |
//! | :--- | :--- |
//! | Super+V | clipboard history |
//! | Super+R | recording controls |
//! | Super+F4 | power menu |

use gio::prelude::*;
use gtk4::prelude::GtkApplicationExt;
use gtk4_layer_shell::{KeyboardMode, LayerShell};

/// (action name, GTK accelerator, trigger topic)
const SHORTCUTS: [(&str, &str, &str); 3] = [
    ("island-clipboard", "<Super>v", "clipboard"),
    ("island-recording", "<Super>r", "recording"),
    ("island-power", "<Super>F4", "power"),
];

/// Registers the fixed accelerators on the running GTK application.
/// Safe to call more than once.
pub fn init(app: &gtk4::Application) {
    if app.lookup_action(SHORTCUTS[0].0).is_some() {
        return;
    }

    for (name, accelerator, topic) in SHORTCUTS {
        let action = gio::SimpleAction::new(name, None);
        action.connect_activate(move |_, _| crate::center::ipc::trigger(topic));
        app.add_action(&action);
        app.set_accels_for_action(&format!("app.{name}"), &[accelerator]);
    }

    // The panel starts with KeyboardMode::None. Switch it to OnDemand once the
    // window is realized so the accelerators can actually be delivered.
    let app = app.downgrade();
    glib::idle_add_local_once(move || {
        if let Some(app) = app.upgrade() {
            if let Some(window) = app.active_window() {
                if window.is_layer_window() {
                    window.set_keyboard_mode(KeyboardMode::OnDemand);
                }
            }
        }
    });
}
