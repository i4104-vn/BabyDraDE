use std::process::Command;
use tracing::debug;

use crate::theme::model::ThemeConfig;

pub fn sync_gsettings(config: &ThemeConfig) {
    let schema = "org.gnome.desktop.interface";

    let settings = [
        ("gtk-theme", config.gtk_theme.as_str()),
        ("icon-theme", config.icon_theme.as_str()),
        ("cursor-theme", config.cursor_theme.as_str()),
        ("font-name", config.font_name.as_str()),
        ("color-scheme", config.color_scheme.as_gsettings_str()),
    ];

    for (key, val) in settings {
        let status = Command::new("gsettings")
            .args(["set", schema, key, val])
            .status();

        match status {
            Ok(s) if s.success() => {
                debug!("gsettings set {} {} succeeded", key, val);
            }
            Ok(s) => {
                debug!("gsettings set {} {} returned exit code: {}", key, val, s);
            }
            Err(err) => {
                // gsettings command not installed or failed to spawn
                debug!("gsettings not available: {}", err);
                return;
            }
        }
    }

    // Set cursor size (integer)
    let size_str = config.cursor_size.to_string();
    let _ = Command::new("gsettings")
        .args(["set", schema, "cursor-size", &size_str])
        .status();
}
