//! Data models for the split BabyDraDE configuration.
//!
//! Main settings live in `~/.babydra/babydra.conf`; subsystem settings live in
//! `~/.babydra/configs/*.toml`. Organized into granular submodules per service.

/// Appearance-related configuration models.
pub mod appearance {
    /// Blur effect configuration model.
    #[path = "blur.rs"]
    pub mod blur;
    /// Cursor configuration model.
    #[path = "cursor.rs"]
    pub mod cursor;
    /// Desktop layout configuration model.
    #[path = "desktop.rs"]
    pub mod desktop;
    /// Display and scaling configuration model.
    #[path = "display.rs"]
    pub mod display;
    /// Wallpaper configuration model.
    #[path = "wallpaper.rs"]
    pub mod wallpaper;
}

/// Input device configuration models.
pub mod input {
    /// Gesture configuration model.
    #[path = "gestures.rs"]
    pub mod gestures;
    /// Mouse configuration model.
    #[path = "mouse.rs"]
    pub mod mouse;
    /// Touchpad configuration model.
    #[path = "touchpad.rs"]
    pub mod touchpad;
}

/// Session configuration models.
pub mod session {
    /// Autostart configuration model.
    #[path = "autostart.rs"]
    pub mod autostart;
    /// Main application configuration model.
    #[path = "babydra.rs"]
    pub mod babydra;
    /// Lock screen configuration model.
    #[path = "lockscreen.rs"]
    pub mod lockscreen;
    /// Notification configuration model.
    #[path = "notification.rs"]
    pub mod notification;
}

/// System behavior configuration models.
pub mod system {
    /// Clipboard behavior configuration model.
    #[path = "clipboard.rs"]
    pub mod clipboard;
    /// Power management configuration model.
    #[path = "power.rs"]
    pub mod power;
}

// Keep the original flat module paths working alongside the topic folders.
pub use appearance::{blur, cursor, desktop, display, wallpaper};
pub use input::{gestures, mouse, touchpad};
pub use session::{autostart, babydra, lockscreen, notification};
pub use system::{clipboard, power};

pub use autostart::AutostartConfig;
pub use babydra::BabyDraConfig;
pub use blur::BlurConfig;
pub use clipboard::ClipboardConfig;
pub use cursor::CursorConfig;
pub use desktop::{CustomContextItem, DesktopConfig};
pub use display::{DisplayConfig, DisplayMonitorSetting, MonitorOrientation};
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
