//! Subsystem configuration data models matching `~/.babydra/babydra.conf`.
//!
//! Organized into granular submodules per desktop service.

pub mod babydra;
pub mod blur;
pub mod clipboard;
pub mod desktop;
pub mod display;
pub mod gestures;
pub mod lockscreen;
pub mod mouse;
pub mod notification;
pub mod power;
pub mod touchpad;
pub mod wallpaper;

pub use babydra::BabyDraConfig;
pub use blur::BlurConfig;
pub use clipboard::ClipboardConfig;
pub use desktop::{CustomContextItem, DesktopConfig};
pub use display::{DisplayConfig, DisplayMonitorSetting};
pub use gestures::GestureConfig;
pub use lockscreen::LockscreenConfig;
pub use mouse::MouseConfig;
pub use notification::NotificationConfig;
pub use power::PowerConfig;
pub use touchpad::TouchpadConfig;
pub use wallpaper::WallpaperConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_touchpad_config_defaults() {
        let config = TouchpadConfig::default();
        assert!(config.tap_to_click);
        assert!(config.tap_drag);
        assert!(!config.tap_drag_lock);
        assert_eq!(config.button_map, "lrm");
        assert!(config.dwt);
        assert!(!config.natural_scroll);
        assert_eq!(config.speed, None);
    }

    #[test]
    fn test_touchpad_config_toml_roundtrip() {
        let toml_str = r#"
[touchpad]
tap_to_click = true
tap_drag = false
tap_drag_lock = true
button_map = "lmr"
dwt = false
natural_scroll = true
speed = 0.5
"#;
        let config: BabyDraConfig = toml::from_str(toml_str).unwrap();
        assert!(config.touchpad.tap_to_click);
        assert!(!config.touchpad.tap_drag);
        assert!(config.touchpad.tap_drag_lock);
        assert_eq!(config.touchpad.button_map, "lmr");
        assert!(!config.touchpad.dwt);
        assert!(config.touchpad.natural_scroll);
        assert_eq!(config.touchpad.speed, Some(0.5));
    }

    #[test]
    fn test_mouse_config_defaults() {
        let config = MouseConfig::default();
        assert!(!config.natural_scroll);
        assert_eq!(config.speed, None);
        assert!(!config.middle_emulation);
    }

    #[test]
    fn test_mouse_config_toml_roundtrip() {
        let toml_str = r#"
[mouse]
natural_scroll = true
speed = -0.2
middle_emulation = true
"#;
        let config: BabyDraConfig = toml::from_str(toml_str).unwrap();
        assert!(config.mouse.natural_scroll);
        assert_eq!(config.mouse.speed, Some(-0.2));
        assert!(config.mouse.middle_emulation);
    }

    #[test]
    fn test_babydra_config_default_roundtrip() {
        let config = BabyDraConfig::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: BabyDraConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }
}
