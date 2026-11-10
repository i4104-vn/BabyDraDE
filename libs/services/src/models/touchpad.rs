//! Touchpad and trackpad hardware configuration models.

use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_button_map() -> String {
    "lrm".to_string()
}

/// Touchpad / trackpad configuration persisted in `~/.babydra/babydra.conf` under `[touchpad]`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TouchpadConfig {
    /// Whether tap-to-click is enabled (1-finger left click, 2-finger right click).
    #[serde(default = "default_true")]
    pub tap_to_click: bool,
    /// Whether tap-and-drag is enabled (tap once then slide to drag).
    #[serde(default = "default_true")]
    pub tap_drag: bool,
    /// Whether tap drag lock is enabled.
    #[serde(default)]
    pub tap_drag_lock: bool,
    /// Mapping of tap fingers to mouse buttons: `"lrm"` (default) or `"lmr"`.
    #[serde(default = "default_button_map")]
    pub button_map: String,
    /// Whether disable-while-typing (DWT) is active.
    #[serde(default = "default_true")]
    pub dwt: bool,
    /// Whether natural (reverse) scrolling direction is enabled.
    #[serde(default)]
    pub natural_scroll: bool,
    /// Optional pointer acceleration speed (-1.0 to 1.0).
    #[serde(default)]
    pub speed: Option<f64>,
}

impl Default for TouchpadConfig {
    /// Returns default touchpad hardware settings.
    ///
    /// # Returns
    ///
    /// A new [`TouchpadConfig`] instance with tap-to-click, tap-drag, and DWT enabled.
    fn default() -> Self {
        Self {
            tap_to_click: true,
            tap_drag: true,
            tap_drag_lock: false,
            button_map: default_button_map(),
            dwt: true,
            natural_scroll: false,
            speed: None,
        }
    }
}
