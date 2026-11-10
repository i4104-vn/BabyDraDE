//! Display and monitor configuration models.

use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_monitor_width() -> u32 {
    1920
}

fn default_monitor_height() -> u32 {
    1080
}

fn default_refresh_rate() -> f64 {
    60.0
}

fn default_orientation() -> String {
    "normal".to_string()
}

fn default_scale() -> f64 {
    1.0
}

/// Settings for an individual physical display output monitor.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DisplayMonitorSetting {
    /// Connector identifier (e.g. `"eDP-1"`, `"HDMI-A-1"`, `"DP-1"`).
    #[serde(default)]
    pub name: String,
    /// Horizontal resolution in physical pixels.
    #[serde(default = "default_monitor_width")]
    pub resolution_width: u32,
    /// Vertical resolution in physical pixels.
    #[serde(default = "default_monitor_height")]
    pub resolution_height: u32,
    /// Display refresh rate in Hertz.
    #[serde(default = "default_refresh_rate")]
    pub refresh_rate: f64,
    /// Position along the virtual canvas X axis in logical pixels.
    #[serde(default)]
    pub position_x: i32,
    /// Position along the virtual canvas Y axis in logical pixels.
    #[serde(default)]
    pub position_y: i32,
    /// Output rotation/orientation (`"normal"`, `"90"`, `"180"`, `"270"`).
    #[serde(default = "default_orientation")]
    pub orientation: String,
    /// Whether this display monitor is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// UI scaling factor for HiDPI displays (e.g. `1.0`, `1.25`, `1.5`, `2.0`).
    #[serde(default = "default_scale")]
    pub scale: f64,
}

impl Default for DisplayMonitorSetting {
    /// Returns default monitor settings (1920x1080@60Hz, 1.0 scale).
    ///
    /// # Returns
    ///
    /// A new [`DisplayMonitorSetting`] instance.
    fn default() -> Self {
        Self {
            name: String::new(),
            resolution_width: default_monitor_width(),
            resolution_height: default_monitor_height(),
            refresh_rate: default_refresh_rate(),
            position_x: 0,
            position_y: 0,
            orientation: default_orientation(),
            enabled: default_true(),
            scale: default_scale(),
        }
    }
}

/// Multi-monitor layout configuration persisted in `~/.babydra/babydra.conf` under `[display]`.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct DisplayConfig {
    /// List of configured display monitor outputs.
    #[serde(default)]
    pub monitors: Vec<DisplayMonitorSetting>,
}
