//! Mouse and pointing device configuration models.

use serde::{Deserialize, Serialize};

/// Mouse / pointer configuration persisted in `~/.babydra/babydra.conf` under `[mouse]`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MouseConfig {
    /// Whether natural (reverse) scrolling direction is enabled for mouse wheels.
    #[serde(default)]
    pub natural_scroll: bool,
    /// Optional pointer acceleration speed (-1.0 to 1.0).
    #[serde(default)]
    pub speed: Option<f64>,
    /// Whether middle mouse button emulation (pressing left and right together) is enabled.
    #[serde(default)]
    pub middle_emulation: bool,
}

impl Default for MouseConfig {
    /// Returns default mouse hardware settings.
    ///
    /// # Returns
    ///
    /// A new [`MouseConfig`] instance with standard scrolling direction and hardware default speed.
    fn default() -> Self {
        Self {
            natural_scroll: false,
            speed: None,
            middle_emulation: false,
        }
    }
}
