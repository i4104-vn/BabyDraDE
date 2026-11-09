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

fn default_blur_radius() -> u32 {
    20
}
fn default_opacity() -> f64 {
    0.75
}
fn default_border_color() -> String {
    "#ffffff".to_string()
}
fn default_border_width() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ThemeSelection {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub dark: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeConfig {
    #[serde(default = "default_blur_radius")]
    pub blur_radius: u32,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default = "default_border_color")]
    pub border_color: String,
    #[serde(default = "default_border_width")]
    pub border_width: u32,
    #[serde(default)]
    pub selection: ThemeSelection,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            blur_radius: default_blur_radius(),
            opacity: default_opacity(),
            border_color: default_border_color(),
            border_width: default_border_width(),
            selection: ThemeSelection::default(),
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
    pub theme: ThemeConfig,
    #[serde(default)]
    pub clipboard: ClipboardConfig,
}
