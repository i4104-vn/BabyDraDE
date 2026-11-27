//! Main configuration model for `~/.babydra/babydra.conf`.
//!
//! Subsystem settings live in `~/.babydra/configs/*.toml`; see [`crate::config`]
//! for the full layout and migration rules.

use serde::{Deserialize, Serialize};

use crate::models::{AutostartConfig, NotificationConfig, PowerConfig, WallpaperConfig};

/// Main configuration model matching `~/.babydra/babydra.conf`.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct BabyDraConfig {
    /// Flag indicating whether this is the initial first-time user login.
    #[serde(rename = "1st", default)]
    pub first_login: bool,
    /// Autostart commands executed on compositor startup.
    #[serde(default)]
    pub autostart: AutostartConfig,
    /// Power profiles and battery threshold settings.
    #[serde(default)]
    pub power: PowerConfig,
    /// Wallpaper path and rendering mode settings.
    #[serde(default)]
    pub wallpaper: WallpaperConfig,
    /// Notification daemon and DND settings.
    #[serde(default)]
    pub notification: NotificationConfig,
}
