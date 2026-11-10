//! Root BabyDra configuration model uniting all subsystem settings.

use serde::{Deserialize, Serialize};

use super::{
    blur::BlurConfig, clipboard::ClipboardConfig, desktop::DesktopConfig, display::DisplayConfig,
    gestures::GestureConfig, lockscreen::LockscreenConfig, mouse::MouseConfig,
    notification::NotificationConfig, power::PowerConfig, touchpad::TouchpadConfig,
    wallpaper::WallpaperConfig,
};

/// Master configuration model matching `~/.babydra/babydra.conf`.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct BabyDraConfig {
    /// Flag indicating whether this is the initial first-time user login.
    #[serde(rename = "1st", default)]
    pub first_login: bool,
    /// Power profiles and battery threshold settings.
    #[serde(default)]
    pub power: PowerConfig,
    /// Wallpaper path and rendering mode settings.
    #[serde(default)]
    pub wallpaper: WallpaperConfig,
    /// Notification daemon and DND settings.
    #[serde(default)]
    pub notification: NotificationConfig,
    /// Multi-monitor layout and display settings.
    #[serde(default)]
    pub display: DisplayConfig,
    /// Lock screen appearance and user avatar settings.
    #[serde(default)]
    pub lockscreen: LockscreenConfig,
    /// Desktop icon grid and custom context menu settings.
    #[serde(default)]
    pub desktop: DesktopConfig,
    /// Clipboard manager and picker hotkey settings.
    #[serde(default)]
    pub clipboard: ClipboardConfig,
    /// Compositor dual-Kawase window blur effect settings.
    #[serde(default)]
    pub blur: BlurConfig,
    /// Touchpad tap-to-click, gestures, and scrolling settings.
    #[serde(default)]
    pub touchpad: TouchpadConfig,
    /// Mouse pointer speed and natural scroll settings.
    #[serde(default)]
    pub mouse: MouseConfig,
    /// Multi-finger swipe and touchscreen gesture settings.
    #[serde(default)]
    pub gestures: GestureConfig,
}
