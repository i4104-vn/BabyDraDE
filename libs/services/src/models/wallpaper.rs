//! Wallpaper configuration models.

use serde::{Deserialize, Serialize};

fn default_wallpaper_mode() -> String {
    "static".to_string()
}

/// Wallpaper configuration persisted in `~/.babydra/babydra.conf` under `[wallpaper]`.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct WallpaperConfig {
    /// Absolute path to the currently active wallpaper image file.
    #[serde(default)]
    pub current: String,
    /// Wallpaper rendering mode (`"static"`, `"slideshow"`, or `"video"`).
    #[serde(default = "default_wallpaper_mode")]
    pub mode: String,
}
