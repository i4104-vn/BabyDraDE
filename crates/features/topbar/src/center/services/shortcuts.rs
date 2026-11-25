//! Island shortcuts registered with the compositor.
//!
//! | Keys | Trigger |
//! | :--- | :--- |
//! | Super+V | clipboard history |
//! | Super+R | recording controls |
//! | Super+F4 | power menu |
//!
//! GTK accelerators on the panel never fired because the compositor consumes
//! these key presses before a layer shell can see them. Publishing the same
//! combos through `services::system::shortcuts` lets the compositor own the
//! bindings and dispatch them to the island's existing D-Bus methods.

use std::sync::atomic::{AtomicBool, Ordering};

use services::system::shortcuts::{ShortcutClient, ShortcutTarget};

const OWNER: &str = "org.babydrade.topbar";
const BUS_NAME: &str = "org.babydra.Island";
const OBJECT_PATH: &str = "/org/babydra/Island";
const INTERFACE: &str = "org.babydra.Island";

/// (shortcut id, combo, island D-Bus method).
const SHORTCUTS: [(&str, &str, &str); 3] = [
    ("clipboard", "W-v", "ToggleClipboard"),
    ("recording", "W-r", "ShowRecording"),
    ("power", "W-F4", "TogglePower"),
];

static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Publishes island shortcuts once; later calls are no-ops.
pub fn init() {
    if INITIALIZED.swap(true, Ordering::SeqCst) {
        return;
    }

    let client = ShortcutClient::new(OWNER);
    for (id, combo, method) in SHORTCUTS {
        if let Err(e) = client.bind(id, combo, island_command(method)) {
            tracing::warn!("cannot register island shortcut {id}: {e}");
        }
    }
    tracing::info!("published {} island shortcuts to the compositor", SHORTCUTS.len());
}

/// Builds the command the compositor runs to trigger one island method.
fn island_command(method: &str) -> ShortcutTarget {
    ShortcutTarget::Command(format!(
        "gdbus call --session --dest {BUS_NAME} --object-path {OBJECT_PATH} --method {INTERFACE}.{method}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn island_command_targets_the_island_bus_name() {
        let ShortcutTarget::Command(command) = island_command("ToggleClipboard") else {
            panic!("island shortcuts must use command targets");
        };
        assert!(command.contains("--dest org.babydra.Island"));
        assert!(command.contains("--method org.babydra.Island.ToggleClipboard"));
    }

    #[test]
    fn combos_are_valid_and_unique() {
        let combos: Vec<_> = SHORTCUTS.iter().map(|(_, combo, _)| *combo).collect();
        assert_eq!(combos.len(), 3);
        let mut sorted = combos.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), combos.len());
        assert!(combos.iter().all(|combo| combo.contains('-')));
    }
}
