//! Mouse and pointer device configuration, natural scrolling, and acceleration.

/// Pointer / mouse configuration options loaded from `~/.babydra/babydra.conf`.
#[derive(Debug, Clone, PartialEq)]
pub struct MouseConfig {
    /// Whether natural (reverse) scrolling direction is enabled for mouse wheels.
    pub natural_scroll: bool,
    /// Optional pointer acceleration speed (-1.0 to 1.0).
    pub speed: Option<f64>,
    /// Whether middle mouse button emulation (pressing left and right together) is enabled.
    pub middle_emulation: bool,
}

impl Default for MouseConfig {
    /// Creates default mouse configuration.
    ///
    /// By default:
    /// - `natural_scroll` is `false`.
    /// - `speed` is `None` (uses hardware default).
    /// - `middle_emulation` is `false`.
    fn default() -> Self {
        Self {
            natural_scroll: false,
            speed: None,
            middle_emulation: false,
        }
    }
}

impl From<services::models::MouseConfig> for MouseConfig {
    /// Converts a persistent [`services::models::MouseConfig`] from `babydra.conf`
    /// into compositor runtime [`MouseConfig`].
    ///
    /// # Arguments
    ///
    /// * `cfg` - The deserialized mouse configuration from `babydra.conf`.
    ///
    /// # Returns
    ///
    /// Returns the converted [`MouseConfig`].
    fn from(cfg: services::models::MouseConfig) -> Self {
        Self {
            natural_scroll: cfg.natural_scroll,
            speed: cfg.speed.map(|s| s.clamp(-1.0, 1.0)),
            middle_emulation: cfg.middle_emulation,
        }
    }
}

impl MouseConfig {
    /// Loads the mouse configuration directly from `~/.babydra/babydra.conf` under `[mouse]`.
    ///
    /// # Returns
    ///
    /// Returns a resolved [`MouseConfig`] mapped from the shared [`services::models::MouseConfig`].
    pub fn load() -> Self {
        let shared_cfg = services::config::load_mouse_config();
        Self::from(shared_cfg)
    }
}

/// Applies hardware configuration settings to a physical mouse or external pointer device.
///
/// Configures:
/// - Natural scrolling (reverse scroll wheel direction)
/// - Pointer acceleration speed
/// - Middle click emulation
///
/// # Arguments
///
/// * `device` - Mutable reference to the [`libinput::Device`](smithay::reexports::input::Device).
/// * `config` - The resolved [`MouseConfig`] containing user preferences.
pub(crate) fn configure_mouse(
    device: &mut smithay::reexports::input::Device,
    config: &MouseConfig,
) {
    use tracing::{info, warn};

    let name = device.name().to_string();

    // 1. Natural scrolling
    if device.config_scroll_has_natural_scroll() {
        match device.config_scroll_set_natural_scroll_enabled(config.natural_scroll) {
            Ok(()) => info!(
                "Mouse '{}': natural scroll set to {}",
                name, config.natural_scroll
            ),
            Err(err) => warn!(
                "Failed to set natural scroll on mouse '{}': {:?}",
                name, err
            ),
        }
    }

    // 2. Pointer acceleration / speed
    if device.config_accel_is_available() {
        if let Some(speed) = config.speed {
            if let Err(err) = device.config_accel_set_speed(speed) {
                warn!("Failed to set speed {} on mouse '{}': {:?}", speed, name, err);
            }
        }
    }

    // 3. Middle click emulation
    if device.config_middle_emulation_is_available() {
        let _ = device.config_middle_emulation_set_enabled(config.middle_emulation);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_config_defaults() {
        let config = MouseConfig::default();
        assert!(!config.natural_scroll);
        assert_eq!(config.speed, None);
        assert!(!config.middle_emulation);
    }

    #[test]
    fn test_mouse_config_from_shared_config() {
        let shared = services::models::MouseConfig {
            natural_scroll: true,
            speed: Some(0.8),
            middle_emulation: true,
        };
        let config = MouseConfig::from(shared);
        assert!(config.natural_scroll);
        assert_eq!(config.speed, Some(0.8));
        assert!(config.middle_emulation);
    }

    #[test]
    fn test_mouse_config_speed_clamping() {
        let shared = services::models::MouseConfig {
            natural_scroll: false,
            speed: Some(5.0),
            middle_emulation: false,
        };
        let config = MouseConfig::from(shared);
        assert_eq!(config.speed, Some(1.0));

        let shared_neg = services::models::MouseConfig {
            natural_scroll: false,
            speed: Some(-5.0),
            middle_emulation: false,
        };
        let config_neg = MouseConfig::from(shared_neg);
        assert_eq!(config_neg.speed, Some(-1.0));
    }
}
