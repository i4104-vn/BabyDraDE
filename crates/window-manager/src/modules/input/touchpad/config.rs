//! Touchpad configuration and device setup.

use self::models::TouchpadConfig;
use smithay::reexports::input::ClickMethod;
use tracing::{debug, info, warn};

pub(crate) fn handle_device_added<D: std::any::Any>(device: &mut D) {
    if let Some(libinput_device) =
        (device as &mut dyn std::any::Any).downcast_mut::<smithay::reexports::input::Device>()
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
        info!(
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
        debug!(
            "Input device '{}' ({}) does not support tapping, configuring as pointer/mouse",
            name, sysname
        );
        let mouse_config = crate::modules::input::pointer::config::MouseConfig::load();
        crate::modules::input::pointer::config::configure_mouse(device, &mouse_config);
    }
}

/// Compatibility alias for the former `config::models` module.
pub(crate) mod models {
    pub(crate) use crate::modules::input::models::TouchpadConfig;
}
