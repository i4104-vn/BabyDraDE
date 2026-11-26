//! Data models for the split BabyDraDE configuration.
//!
//! Main settings live in `~/.babydra/babydra.conf`; subsystem settings live in
//! `~/.babydra/configs/*.toml`. Organized into granular submodules per service.

pub mod autostart;
/// Main application configuration model.
pub mod babydra;
/// Blur effect configuration model.
pub mod blur;
/// Clipboard behavior configuration model.
pub mod clipboard;
/// Cursor configuration model.
pub mod cursor;
/// Desktop layout configuration model.
pub mod desktop;
/// Display and scaling configuration model.
pub mod display;
/// Gesture configuration model.
pub mod gestures;
/// Lock screen configuration model.
pub mod lockscreen;
/// Mouse configuration model.
pub mod mouse;
/// Notification configuration model.
pub mod notification;
/// Power management configuration model.
pub mod power;
/// Touchpad configuration model.
pub mod touchpad;
/// Wallpaper configuration model.
pub mod wallpaper;

pub use autostart::AutostartConfig;
pub use babydra::BabyDraConfig;
pub use blur::BlurConfig;
pub use clipboard::ClipboardConfig;
pub use cursor::CursorConfig;
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
    fn test_touchpad_config_toml_roundtrip() {
        let toml_str = r#"
tap_to_click = true
tap_drag = false
tap_drag_lock = true
button_map = "lmr"
dwt = false
natural_scroll = true
speed = 0.5
"#;
        let config: TouchpadConfig = toml::from_str(toml_str).unwrap();
        assert!(config.tap_to_click);
        assert!(!config.tap_drag);
        assert!(config.tap_drag_lock);
        assert_eq!(config.button_map, "lmr");
        assert!(!config.dwt);
        assert!(config.natural_scroll);
        assert_eq!(config.speed, Some(0.5));
    }

    #[test]
    fn test_mouse_config_toml_roundtrip() {
        let toml_str = r#"
natural_scroll = true
speed = -0.2
middle_emulation = true
"#;
        let config: MouseConfig = toml::from_str(toml_str).unwrap();
        assert!(config.natural_scroll);
        assert_eq!(config.speed, Some(-0.2));
        assert!(config.middle_emulation);
    }

    #[test]
    fn test_cursor_config_toml_roundtrip() {
        let toml_str = r#"
theme = "Breeze"
size = 32
hotspot_x = 2
hotspot_y = 1
"#;
        let config: CursorConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.theme, "Breeze");
        assert_eq!(config.size, 32);
        assert_eq!(config.hotspot_x, 2);
        assert_eq!(config.hotspot_y, 1);
    }

    #[test]
    fn test_babydra_config_default_roundtrip() {
        let config = BabyDraConfig::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: BabyDraConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_main_config_omits_subsystem_sections() {
        let serialized = toml::to_string(&BabyDraConfig::default()).unwrap();
        for section in [
            "[blur]",
            "[clipboard]",
            "[cursor]",
            "[desktop]",
            "[display]",
            "[gestures]",
            "[lockscreen]",
            "[mouse]",
            "[touchpad]",
        ] {
            assert!(
                !serialized.contains(section),
                "main config must not contain {section}"
            );
        }
    }
}
