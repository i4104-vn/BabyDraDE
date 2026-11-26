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

/// Fixed rotation of a physical display output.
///
/// The string form matches both the config value and `wlr-randr --transform`,
/// so the compositor and the greeter share one vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonitorOrientation {
    /// No rotation (`normal` or `0`).
    Normal,
    /// Quarter turn clockwise (`90`).
    Rotate90,
    /// Half turn (`180`).
    Rotate180,
    /// Three quarter turns clockwise (`270`).
    Rotate270,
    /// Horizontally mirrored (`flipped`).
    Flipped,
    /// Mirrored then rotated (`flipped-90`).
    Flipped90,
    /// Mirrored then half turned (`flipped-180`).
    Flipped180,
    /// Mirrored then rotated three quarters (`flipped-270`).
    Flipped270,
}

impl MonitorOrientation {
    /// Parses a configured orientation value.
    ///
    /// # Arguments
    /// * `value` - Config value such as `normal`, `90` or `flipped-90`.
    ///
    /// # Returns
    /// The matching orientation, or `None` when the value is unknown.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "normal" | "0" => Some(Self::Normal),
            "90" | "left" => Some(Self::Rotate90),
            "180" | "inverted" => Some(Self::Rotate180),
            "270" | "right" => Some(Self::Rotate270),
            "flipped" => Some(Self::Flipped),
            "flipped-90" => Some(Self::Flipped90),
            "flipped-180" => Some(Self::Flipped180),
            "flipped-270" => Some(Self::Flipped270),
            _ => None,
        }
    }

    /// Returns the config/`wlr-randr` string form of this orientation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Rotate90 => "90",
            Self::Rotate180 => "180",
            Self::Rotate270 => "270",
            Self::Flipped => "flipped",
            Self::Flipped90 => "flipped-90",
            Self::Flipped180 => "flipped-180",
            Self::Flipped270 => "flipped-270",
        }
    }
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

impl DisplayMonitorSetting {
    /// Returns the parsed output rotation, if the configured value is valid.
    pub fn parsed_orientation(&self) -> Option<MonitorOrientation> {
        MonitorOrientation::parse(&self.orientation)
    }
}

/// Multi-monitor layout configuration persisted in
/// `~/.babydra/configs/display.toml` under `[display]`.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct DisplayConfig {
    /// List of configured display monitor outputs.
    #[serde(default)]
    pub monitors: Vec<DisplayMonitorSetting>,
    /// Follow the device accelerometer and rotate the panel automatically.
    /// `None` (the default) enables it whenever a sensor is detected; set to
    /// `false` to opt out on a convertible machine.
    #[serde(default)]
    pub auto_rotate: Option<bool>,
    /// Preferred accelerometer driver: `auto` (the default), `gaokun3`, `iio`
    /// or `sensor-proxy`. Kept as a free-form string so new drivers can be added
    /// to the compositor without changing the config schema. Unknown names
    /// fall back to automatic detection.
    #[serde(default)]
    pub auto_rotate_driver: Option<String>,
}

impl DisplayConfig {
    /// Returns the fixed orientation configured for a physical connector.
    ///
    /// Connector names are compared case-insensitively because DRM reports
    /// uppercase names (`eDP-1`) while users often write them lowercase.
    ///
    /// # Arguments
    /// * `connector` - Output name reported by DRM or `wlr-randr`.
    ///
    /// # Returns
    /// The configured orientation, or `None` when absent, disabled or invalid.
    pub fn orientation_for(&self, connector: &str) -> Option<MonitorOrientation> {
        self.monitors
            .iter()
            .find(|monitor| monitor.enabled && monitor.name.eq_ignore_ascii_case(connector))
            .and_then(DisplayMonitorSetting::parsed_orientation)
    }

    /// Returns the fixed orientation of the first enabled configured monitor.
    ///
    /// Used by the single-output nested backend, which has no physical connector
    /// name to match against.
    pub fn primary_orientation(&self) -> Option<MonitorOrientation> {
        self.monitors
            .iter()
            .find(|monitor| monitor.enabled)
            .and_then(DisplayMonitorSetting::parsed_orientation)
    }
}

#[cfg(test)]
mod tests {
    use super::{DisplayConfig, DisplayMonitorSetting, MonitorOrientation};

    #[test]
    fn parsed_orientation_accepts_config_and_wlr_randr_values() {
        assert_eq!(
            MonitorOrientation::parse("  FLIPPED-90 "),
            Some(MonitorOrientation::Flipped90)
        );
        assert_eq!(MonitorOrientation::parse("0"), Some(MonitorOrientation::Normal));
        assert_eq!(MonitorOrientation::parse("270"), Some(MonitorOrientation::Rotate270));
        assert_eq!(MonitorOrientation::parse("sideways"), None);
    }

    #[test]
    fn orientation_for_ignores_disabled_and_unknown_monitors() {
        let matching = DisplayMonitorSetting {
            name: "edp-1".to_string(),
            orientation: "90".to_string(),
            ..DisplayMonitorSetting::default()
        };
        let disabled = DisplayMonitorSetting {
            name: "HDMI-A-1".to_string(),
            orientation: "270".to_string(),
            enabled: false,
            ..DisplayMonitorSetting::default()
        };

        let config = DisplayConfig {
            monitors: vec![matching, disabled],
            ..DisplayConfig::default()
        };

        assert_eq!(
            config.orientation_for("eDP-1"),
            Some(MonitorOrientation::Rotate90)
        );
        assert_eq!(config.orientation_for("HDMI-A-1"), None);
        assert_eq!(config.orientation_for("DP-1"), None);
    }
}
