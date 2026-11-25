//! Hardcoded island shortcuts — no configuration file involved.
//!
//! | Keys | Trigger |
//! | :--- | :--- |
//! | Super+V | clipboard history |
//! | Super+R | recording controls |
//! | Super+F4 | power menu |

use gio::prelude::*;
use gtk4::prelude::{GtkApplicationExt, WidgetExt};
use gtk4_layer_shell::{KeyboardMode, LayerShell};

/// (action name, GTK accelerator, trigger topic)
const SHORTCUTS: [(&str, &str, &str); 3] = [
    ("island-clipboard", "<Super>v", "clipboard"),
    ("island-recording", "<Super>r", "recording"),
    ("island-power", "<Super>F4", "power"),
];

/// Registers the fixed accelerators on the running GTK application.
/// Safe to call more than once.
pub fn init(capsule: &gtk4::Box) {
    // Layer-shell panels start with KeyboardMode::None; accelerators need the
    // window to accept key events, so switch it to OnDemand once.
    if let Some(root) = capsule.root() {
        if let Some(window) = root.downcast_ref::<gtk4::Window>() {
            if window.is_layer_window() {
                window.set_keyboard_mode(KeyboardMode::OnDemand);
            }
        }
    }

    let app = gtk4::Application::default();
    if app.lookup_action(SHORTCUTS[0].0).is_some() {
        return;
    }

    for (name, accelerator, topic) in SHORTCUTS {
        let action = gio::SimpleAction::new(name, None);
        action.connect_activate(move |_, _| crate::center::ipc::trigger(topic));
        app.add_action(&action);
        app.set_accels_for_action(&format!("app.{name}"), &[accelerator]);
    }
}
