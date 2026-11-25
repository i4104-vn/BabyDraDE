//! Mouse and pointer device configuration, natural scrolling, and acceleration.

use tracing::{info, warn};

pub(crate) use crate::modules::input::models::MouseConfig;

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
                warn!(
                    "Failed to set speed {} on mouse '{}': {:?}",
                    speed, name, err
                );
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
