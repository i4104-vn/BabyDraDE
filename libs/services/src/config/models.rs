//! Configuration data models matching babydra-core.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PowerConfig {
    #[serde(default = "default_power_profile")]
    pub profile: String,
    #[serde(default = "default_true")]
    pub auto_saver_enabled: bool,
    #[serde(default = "default_saver_threshold")]
    pub saver_threshold: u32,
    #[serde(default = "default_charge_limit")]
    pub charge_limit: u32,
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            profile: default_power_profile(),
            auto_saver_enabled: default_true(),
            saver_threshold: default_saver_threshold(),
            charge_limit: default_charge_limit(),
        }
    }
}

fn default_wallpaper_mode() -> String {
    "static".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct WallpaperConfig {
    #[serde(default)]
    pub current: String,
    #[serde(default = "default_wallpaper_mode")]
    pub mode: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct NotificationConfig {
    #[serde(default)]
    pub dnd: bool,
}

fn default_monitor_width() -> u32 {
    1920
}
fn default_monitor_height() -> u32 {
    1080
}
fn default_refresh_rate() -> f64 {
    60.0
}
fn default_orientation() -> String {
    "normal".to_string()
}
fn default_scale() -> f64 {
    1.0
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DisplayMonitorSetting {
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_monitor_width")]
    pub resolution_width: u32,
    #[serde(default = "default_monitor_height")]
    pub resolution_height: u32,
    #[serde(default = "default_refresh_rate")]
    pub refresh_rate: f64,
    #[serde(default)]
    pub position_x: i32,
    #[serde(default)]
    pub position_y: i32,
    #[serde(default = "default_orientation")]
    pub orientation: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_scale")]
    pub scale: f64,
}

impl Default for DisplayMonitorSetting {
    fn default() -> Self {
        Self {
            name: String::new(),
            resolution_width: default_monitor_width(),
            resolution_height: default_monitor_height(),
            refresh_rate: default_refresh_rate(),
            position_x: 0,
            position_y: 0,
            orientation: default_orientation(),
            enabled: default_true(),
            scale: default_scale(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct DisplayConfig {
    #[serde(default)]
    pub monitors: Vec<DisplayMonitorSetting>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct LockscreenConfig {
    #[serde(default)]
    pub background: String,
    #[serde(default)]
    pub avatar: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CustomContextItem {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub icon: Option<String>,
}

fn default_icon_size() -> u32 {
    48
}
fn default_grid_spacing() -> u32 {
    16
}
fn default_desktop_sort_by() -> String {
    "name".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DesktopConfig {
    #[serde(default = "default_true")]
    pub show_icons: bool,
    #[serde(default = "default_icon_size")]
    pub icon_size: u32,
    #[serde(default = "default_grid_spacing")]
    pub grid_spacing: u32,
    #[serde(default = "default_desktop_sort_by")]
    pub sort_by: String,
    #[serde(default = "default_true")]
    pub auto_arrange: bool,
    #[serde(skip)]
    pub icon_positions: HashMap<String, (i32, i32)>,
    #[serde(default)]
    pub custom_context_items: Vec<CustomContextItem>,
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            show_icons: true,
            icon_size: default_icon_size(),
            grid_spacing: default_grid_spacing(),
            sort_by: default_desktop_sort_by(),
            auto_arrange: true,
            icon_positions: HashMap::new(),
            custom_context_items: Vec::new(),
        }
    }
}

fn default_clipboard_max_items() -> usize {
    20
}
fn default_clipboard_modifiers() -> String {
    "W".to_string()
}
fn default_clipboard_key() -> String {
    "v".to_string()
}

fn default_blur_radius() -> f32 {
    18.0
}
fn default_blur_passes() -> usize {
    3
}
fn default_blur_corner_radius() -> f32 {
    14.0
}
fn default_blur_window_opacity() -> f32 {
    0.68
}
fn default_blur_tint() -> [f32; 4] {
    [0.10, 0.12, 0.16, 0.18]
}
fn default_blur_noise() -> f32 {
    0.012
}
fn default_blur_brightness() -> f32 {
    1.0
}
fn default_blur_contrast() -> f32 {
    1.0
}
fn default_blur_app_rules() -> Vec<String> {
    vec![
        "foot".to_string(),
        "alacritty".to_string(),
        "kitty".to_string(),
        "wezterm".to_string(),
        "org.gnome.Terminal".to_string(),
        "terminal".to_string(),
        "babydra-panel".to_string(),
        "code".to_string(),
        "nautilus".to_string(),
        "thunar".to_string(),
    ]
}

/// Shared compositor blur configuration persisted in `~/.babydra/babydra.conf`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BlurConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub blur_all: bool,
    #[serde(default = "default_blur_radius")]
    pub radius: f32,
    #[serde(default = "default_blur_passes")]
    pub passes: usize,
    #[serde(default = "default_blur_corner_radius")]
    pub corner_radius: f32,
    #[serde(default = "default_blur_window_opacity")]
    pub window_opacity: f32,
    #[serde(default = "default_blur_tint")]
    pub tint: [f32; 4],
    #[serde(default = "default_blur_noise")]
    pub noise: f32,
    #[serde(default = "default_blur_brightness")]
    pub brightness: f32,
    #[serde(default = "default_blur_contrast")]
    pub contrast: f32,
    #[serde(default = "default_blur_app_rules")]
    pub app_rules: Vec<String>,
}

impl Default for BlurConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            blur_all: false,
            radius: default_blur_radius(),
            passes: default_blur_passes(),
            corner_radius: default_blur_corner_radius(),
            window_opacity: default_blur_window_opacity(),
            tint: default_blur_tint(),
            noise: default_blur_noise(),
            brightness: default_blur_brightness(),
            contrast: default_blur_contrast(),
            app_rules: default_blur_app_rules(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClipboardConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_clipboard_max_items")]
    pub max_items: usize,
    #[serde(default = "default_clipboard_modifiers")]
    pub shortcut_modifiers: String,
    #[serde(default = "default_clipboard_key")]
    pub shortcut_key: String,
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_items: default_clipboard_max_items(),
            shortcut_modifiers: default_clipboard_modifiers(),
            shortcut_key: default_clipboard_key(),
        }
    }
}

fn default_button_map() -> String {
    "lrm".to_string()
}

fn default_swipe_threshold() -> f64 {
    100.0
}
fn default_four_finger_swipe_threshold() -> f64 {
    120.0
}
fn default_touch_swipe_threshold() -> f64 {
    80.0
}
fn default_touch_tap_timeout_ms() -> u64 {
    200
}

/// Gesture configuration persisted in `~/.babydra/babydra.conf` under `[gestures]`.
///
/// Controls which multi-finger swipe and touch gestures are active, and
/// their activation distance thresholds.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GestureConfig {
    /// Enable 3-finger horizontal swipe to cycle window focus.
    #[serde(default = "default_true")]
    pub three_finger_swipe_window: bool,
    /// Enable 3-finger swipe up to maximize the active window.
    #[serde(default = "default_true")]
    pub three_finger_swipe_up_maximize: bool,
    /// Enable 3-finger swipe down to minimize the active window.
    #[serde(default = "default_true")]
    pub three_finger_swipe_down_minimize: bool,
    /// Enable 4-finger horizontal swipe to cycle window focus.
    #[serde(default = "default_true")]
    pub four_finger_swipe_window: bool,
    /// Enable 4-finger swipe down to close the active window.
    #[serde(default = "default_true")]
    pub four_finger_swipe_down_close: bool,
    /// Enable 4-finger swipe up to toggle fullscreen on the active window.
    #[serde(default = "default_true")]
    pub four_finger_swipe_up_fullscreen: bool,
    /// Minimum pixel distance for a 3-finger swipe to activate (default 100.0).
    #[serde(default = "default_swipe_threshold")]
    pub swipe_threshold: f64,
    /// Minimum pixel distance for a 4-finger swipe to activate (default 120.0).
    #[serde(default = "default_four_finger_swipe_threshold")]
    pub four_finger_swipe_threshold: f64,
    /// Enable 2-finger touch swipe left/right to cycle window focus on touchscreens.
    #[serde(default = "default_true")]
    pub touch_two_finger_swipe_window: bool,
    /// Enable 2-finger touch swipe down to minimize the active window.
    #[serde(default = "default_true")]
    pub touch_two_finger_swipe_down_minimize: bool,
    /// Enable 3-finger touch tap to send a middle-click.
    #[serde(default = "default_true")]
    pub touch_three_finger_tap: bool,
    /// Minimum pixel distance for a 2-finger touch swipe to activate (default 80.0).
    #[serde(default = "default_touch_swipe_threshold")]
    pub touch_swipe_threshold: f64,
    /// Maximum time in milliseconds from first contact to lift for a tap (default 200).
    #[serde(default = "default_touch_tap_timeout_ms")]
    pub touch_tap_timeout_ms: u64,
}

impl Default for GestureConfig {
    fn default() -> Self {
        Self {
            three_finger_swipe_window: true,
            three_finger_swipe_up_maximize: true,
            three_finger_swipe_down_minimize: true,
            four_finger_swipe_window: true,
            four_finger_swipe_down_close: true,
            four_finger_swipe_up_fullscreen: true,
            swipe_threshold: default_swipe_threshold(),
            four_finger_swipe_threshold: default_four_finger_swipe_threshold(),
            touch_two_finger_swipe_window: true,
            touch_two_finger_swipe_down_minimize: true,
            touch_three_finger_tap: true,
            touch_swipe_threshold: default_touch_swipe_threshold(),
            touch_tap_timeout_ms: default_touch_tap_timeout_ms(),
        }
    }
}

/// Touchpad / trackpad configuration persisted in `~/.babydra/babydra.conf` under `[touchpad]`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TouchpadConfig {
    /// Whether tap-to-click is enabled (1-finger left click, 2-finger right click).
    #[serde(default = "default_true")]
    pub tap_to_click: bool,
    /// Whether tap-and-drag is enabled (tap once then slide to drag).
    #[serde(default = "default_true")]
    pub tap_drag: bool,
    /// Whether tap drag lock is enabled.
    #[serde(default)]
    pub tap_drag_lock: bool,
    /// Mapping of tap fingers to mouse buttons: `"lrm"` (default) or `"lmr"`.
    #[serde(default = "default_button_map")]
    pub button_map: String,
    /// Whether disable-while-typing (DWT) is active.
    #[serde(default = "default_true")]
    pub dwt: bool,
    /// Whether natural (reverse) scrolling direction is enabled.
    #[serde(default)]
    pub natural_scroll: bool,
    /// Optional pointer acceleration speed (-1.0 to 1.0).
    #[serde(default)]
    pub speed: Option<f64>,
}

impl Default for TouchpadConfig {
    fn default() -> Self {
        Self {
            tap_to_click: true,
            tap_drag: true,
            tap_drag_lock: false,
            button_map: default_button_map(),
            dwt: true,
            natural_scroll: false,
            speed: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct BabyDraConfig {
    #[serde(rename = "1st", default)]
    pub first_login: bool,
    #[serde(default)]
    pub power: PowerConfig,
    #[serde(default)]
    pub wallpaper: WallpaperConfig,
    #[serde(default)]
    pub notification: NotificationConfig,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub lockscreen: LockscreenConfig,
    #[serde(default)]
    pub desktop: DesktopConfig,
    #[serde(default)]
    pub clipboard: ClipboardConfig,
    #[serde(default)]
    pub blur: BlurConfig,
    #[serde(default)]
    pub touchpad: TouchpadConfig,
    #[serde(default)]
    pub gestures: GestureConfig,
}

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
}
