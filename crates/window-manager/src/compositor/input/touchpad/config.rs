//! Touchpad device configuration, tap-to-click settings, and hotplug management.

//! Touchpad device configuration, tap-to-click settings, and hotplug management.

/// Touchpad configuration options loaded from `~/.babydra/babydra.conf`.
#[derive(Debug, Clone, PartialEq)]
pub struct TouchpadConfig {
    /// Whether tap-to-click is enabled (1-finger left click, 2-finger right click).
    pub tap_to_click: bool,
    /// Whether tap-and-drag is enabled (tap once then slide to drag).
    pub tap_drag: bool,
    /// Whether tap drag lock is enabled.
    pub tap_drag_lock: bool,
    /// Mapping of tap finger count to mouse buttons.
    pub button_map: smithay::reexports::input::TapButtonMap,
    /// Whether disable-while-typing (DWT) is active.
    pub dwt: bool,
    /// Whether natural (reverse) scrolling direction is enabled.
    pub natural_scroll: bool,
    /// Optional pointer acceleration speed (-1.0 to 1.0).
    pub speed: Option<f64>,
}

impl Default for TouchpadConfig {
    /// Creates default touchpad configuration suitable for modern laptops.
    ///
    /// By default:
    /// - `tap_to_click` is `true`.
    /// - `tap_drag` is `true`.
    /// - `tap_drag_lock` is `false`.
    /// - `button_map` is `LeftRightMiddle` (1 finger = Left, 2 fingers = Right, 3 fingers = Middle).
    /// - `dwt` is `true`.
    /// - `natural_scroll` is `false`.
    /// - `speed` is `None` (uses hardware default).
    fn default() -> Self {
        Self {
            tap_to_click: true,
            tap_drag: true,
            tap_drag_lock: false,
            button_map: smithay::reexports::input::TapButtonMap::LeftRightMiddle,
            dwt: true,
            natural_scroll: false,
            speed: None,
        }
    }
}

impl From<services::models::TouchpadConfig> for TouchpadConfig {
    /// Converts a persistent [`services::models::TouchpadConfig`] from `babydra.conf`
    /// into compositor runtime [`TouchpadConfig`] with Smithay input types.
    ///
    /// # Arguments
    ///
    /// * `cfg` - The deserialized touchpad configuration from `babydra.conf`.
    ///
    /// # Returns
    ///
    /// Returns the converted [`TouchpadConfig`].
    fn from(cfg: services::models::TouchpadConfig) -> Self {
        let button_map = match cfg.button_map.as_str() {
            "lmr" => smithay::reexports::input::TapButtonMap::LeftMiddleRight,
            _ => smithay::reexports::input::TapButtonMap::LeftRightMiddle,
        };

        Self {
            tap_to_click: cfg.tap_to_click,
            tap_drag: cfg.tap_drag,
            tap_drag_lock: cfg.tap_drag_lock,
            button_map,
            dwt: cfg.dwt,
            natural_scroll: cfg.natural_scroll,
            speed: cfg.speed.map(|s| s.clamp(-1.0, 1.0)),
        }
    }
}

impl TouchpadConfig {
    /// Loads the touchpad configuration directly from `~/.babydra/babydra.conf` under `[touchpad]`.
    ///
    /// # Returns
    ///
    /// Returns a resolved [`TouchpadConfig`] mapped from the shared [`services::models::TouchpadConfig`].
    pub fn load() -> Self {
        let shared_cfg = services::config::load_touchpad_config();
        Self::from(shared_cfg)
    }
}

/// Discovers newly added input devices and configures them if they are touchpads.
///
/// Safely downcasts generic backend devices to [`libinput::Device`](smithay::reexports::input::Device)
/// and applies configuration if supported. Non-libinput devices are safely ignored.
///
/// # Arguments
///
/// * `device` - Mutable reference to the newly connected input device.
pub(crate) fn handle_device_added<D: std::any::Any>(device: &mut D) {
    if let Some(libinput_device) = (device as &mut dyn std::any::Any)
        .downcast_mut::<smithay::reexports::input::Device>()
    {
        let config = TouchpadConfig::load();
        configure_touchpad(libinput_device, &config);
    }
}

/// Handles cleanup and logs information when an input device is removed or unplugged.
///
/// # Arguments
///
/// * `device` - Reference to the disconnected input device.
pub(crate) fn handle_device_removed<D: std::any::Any>(device: &D) {
    if let Some(libinput_device) =
        (device as &dyn std::any::Any).downcast_ref::<smithay::reexports::input::Device>()
    {
        tracing::info!(
            "Input device removed: '{}' ({})",
            libinput_device.name(),
            libinput_device.sysname()
        );
    }
}

/// Applies hardware configuration settings to a physical touchpad/trackpad device.
///
/// Configures:
/// - Tap-to-click (1-finger = left, 2-finger = right, 3-finger = middle)
/// - Tap-and-drag (tap once then slide to drag)
/// - Click method (Clickfinger: 2-finger physical click anywhere on pad = right click)
/// - Disable-while-typing (DWT)
/// - Natural scrolling
/// - Middle click emulation
/// - Pointer acceleration speed
///
/// # Arguments
///
/// * `device` - Mutable reference to the [`libinput::Device`](smithay::reexports::input::Device).
/// * `config` - The resolved [`TouchpadConfig`] containing user preferences.
pub(crate) fn configure_touchpad(
    device: &mut smithay::reexports::input::Device,
    config: &TouchpadConfig,
) {
    use smithay::reexports::input::ClickMethod;
    use tracing::{info, warn};

    let name = device.name().to_string();
    let sysname = device.sysname().to_string();

    let tap_finger_count = device.config_tap_finger_count();
    if tap_finger_count > 0 {
        info!(
            "Trackpad detected: '{}' ({}), supported tap fingers: {}",
            name, sysname, tap_finger_count
        );

        // 1. Tap-to-click (enabled by default so users can tap without pressing physical buttons)
        if config.tap_to_click {
            match device.config_tap_set_enabled(true) {
                Ok(()) => info!("Tap-to-click successfully enabled on '{}'", name),
                Err(err) => warn!("Failed to enable tap-to-click on '{}': {:?}", name, err),
            }

            // 2. Tap-and-drag (tap then hold and move to drag)
            if let Err(err) = device.config_tap_set_drag_enabled(config.tap_drag) {
                warn!("Failed to set tap-and-drag on '{}': {:?}", name, err);
            }

            // 3. Tap drag lock
            if let Err(err) = device.config_tap_set_drag_lock_enabled(config.tap_drag_lock) {
                warn!("Failed to set tap drag lock on '{}': {:?}", name, err);
            }

            // 4. Tap button map (1 finger = Left, 2 fingers = Right, 3 fingers = Middle)
            if let Err(err) = device.config_tap_set_button_map(config.button_map) {
                warn!("Failed to set tap button map on '{}': {:?}", name, err);
            }
        } else {
            let _ = device.config_tap_set_enabled(false);
            info!(
                "Tap-to-click disabled on '{}' via BABYDRADE_TAP_TO_CLICK=0",
                name
            );
        }

        // 5. Click method: clickfinger (1 finger = left click, 2 fingers = right click anywhere on pad)
        if device
            .config_click_methods()
            .contains(&ClickMethod::Clickfinger)
        {
            if let Err(err) = device.config_click_set_method(ClickMethod::Clickfinger) {
                warn!("Failed to set Clickfinger method on '{}': {:?}", name, err);
            }
        }

        // 6. Disable while typing (DWT)
        if device.config_dwt_is_available() {
            let _ = device.config_dwt_set_enabled(config.dwt);
        }

        // 7. Natural scrolling
        if device.config_scroll_has_natural_scroll() {
            match device.config_scroll_set_natural_scroll_enabled(config.natural_scroll) {
                Ok(()) => info!(
                    "Touchpad '{}': natural scroll set to {}",
                    name, config.natural_scroll
                ),
                Err(err) => warn!(
                    "Failed to set natural scroll on touchpad '{}': {:?}",
                    name, err
                ),
            }
        }

        // 8. Middle click emulation
        if device.config_middle_emulation_is_available() {
            let _ = device.config_middle_emulation_set_enabled(true);
        }

        // 9. Pointer acceleration / speed
        if device.config_accel_is_available() {
            if let Some(speed) = config.speed {
                let _ = device.config_accel_set_speed(speed);
            }
        }
    } else {
        tracing::debug!(
            "Input device '{}' ({}) does not support tapping, configuring as pointer/mouse",
            name,
            sysname
        );
        let mouse_config = crate::compositor::input::pointer::MouseConfig::load();
        crate::compositor::input::pointer::configure_mouse(device, &mouse_config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::reexports::input::TapButtonMap;

    #[test]
    fn test_touchpad_config_defaults() {
        let config = TouchpadConfig::default();
        assert!(config.tap_to_click);
        assert!(config.tap_drag);
        assert!(!config.tap_drag_lock);
        assert_eq!(config.button_map, TapButtonMap::LeftRightMiddle);
        assert!(config.dwt);
        assert!(!config.natural_scroll);
        assert_eq!(config.speed, None);
    }

    #[test]
    fn test_touchpad_config_from_shared_config() {
        let shared = services::models::TouchpadConfig {
            tap_to_click: true,
            tap_drag: false,
            tap_drag_lock: true,
            button_map: "lmr".to_string(),
            dwt: false,
            natural_scroll: true,
            speed: Some(0.75),
        };
        let config = TouchpadConfig::from(shared);
        assert!(config.tap_to_click);
        assert!(!config.tap_drag);
        assert!(config.tap_drag_lock);
        assert_eq!(config.button_map, TapButtonMap::LeftMiddleRight);
        assert!(!config.dwt);
        assert!(config.natural_scroll);
        assert_eq!(config.speed, Some(0.75));
    }

    #[test]
    fn test_handle_device_non_libinput() {
        struct MockDevice {
            _id: u32,
        }
        let mut mock = MockDevice { _id: 42 };
        // Should not panic on non-libinput devices (e.g. Winit devices)
        handle_device_added(&mut mock);
        handle_device_removed(&mock);
    }
}

