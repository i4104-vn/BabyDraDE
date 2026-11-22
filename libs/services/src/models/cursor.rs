//! Cursor appearance configuration persisted in `~/.babydra/babydra.conf`.

use serde::{Deserialize, Serialize};

fn default_theme() -> String {
    "Adwaita".to_string()
}

fn default_size() -> u32 {
    24
}

/// Pointer theme selection for the compositor.
///
/// Themes are looked up in `~/.babydra/cursor/<theme>/cursors/` first, then in
/// the standard XDG icon locations. Both Xcursor files (with no extension) and
/// PNG images (with a `.png` extension) are supported inside that directory.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CursorConfig {
    /// Name of the installed cursor theme directory.
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Preferred cursor size in pixels. Xcursor themes pick the closest size
    /// available; PNG images are used at their native resolution.
    #[serde(default = "default_size")]
    pub size: u32,

    /// Hotspot X offset used for PNG cursors (Xcursor files carry their own).
    #[serde(default)]
    pub hotspot_x: u32,

    /// Hotspot Y offset used for PNG cursors (Xcursor files carry their own).
    #[serde(default)]
    pub hotspot_y: u32,
}

impl Default for CursorConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            size: default_size(),
            hotspot_x: 0,
            hotspot_y: 0,
        }
    }
}
