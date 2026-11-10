//! Power management configuration models.

use serde::{Deserialize, Serialize};

fn default_power_profile() -> String {
    "balanced".to_string()
}

fn default_true() -> bool {
    true
}

fn default_saver_threshold() -> u32 {
    20
}

fn default_charge_limit() -> u32 {
    80
}

/// Power management configuration persisted in `~/.babydra/babydra.conf` under `[power]`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PowerConfig {
    /// Active system power profile (`"performance"`, `"balanced"`, or `"power-saver"`).
    #[serde(default = "default_power_profile")]
    pub profile: String,
    /// Whether auto battery saver triggers when battery drops below [`saver_threshold`].
    #[serde(default = "default_true")]
    pub auto_saver_enabled: bool,
    /// Percentage threshold below which battery saver activates.
    #[serde(default = "default_saver_threshold")]
    pub saver_threshold: u32,
    /// Maximum battery charge limit percentage for longevity.
    #[serde(default = "default_charge_limit")]
    pub charge_limit: u32,
}

impl Default for PowerConfig {
    /// Returns default power configuration values.
    ///
    /// # Returns
    ///
    /// A new [`PowerConfig`] with balanced profile, auto saver enabled at 20%, and 80% charge limit.
    fn default() -> Self {
        Self {
            profile: default_power_profile(),
            auto_saver_enabled: default_true(),
            saver_threshold: default_saver_threshold(),
            charge_limit: default_charge_limit(),
        }
    }
}
