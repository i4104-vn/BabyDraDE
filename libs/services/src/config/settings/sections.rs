//! Typed load/save helpers for each configuration section.

use super::paths::*;
use super::setup::{save_root_or_log, save_section_or_log};
use crate::config::store;
use crate::models::{
    AutostartConfig, BabyDraConfig, BlurConfig, ClipboardConfig, CursorConfig, DesktopConfig,
    DisplayConfig, GestureConfig, LockscreenConfig, MouseConfig, NotificationConfig, PowerConfig,
    TouchpadConfig, WallpaperConfig,
};

/// Loads the main config from `~/.babydra/babydra.conf`.
pub fn load_babydra_config() -> BabyDraConfig {
    store::load_root(&get_conf_path())
}

/// Persists the main config. Subsystem sections are intentionally not written.
pub fn save_babydra_config(config: &BabyDraConfig) {
    save_root_or_log(&get_conf_path(), config, "main");
}

/// Loads `[display]` from `configs/display.toml`.
pub fn load_display_config() -> DisplayConfig {
    store::load_section(&get_display_config_path(), "display")
}

/// Persists `[display]` into `configs/display.toml`.
pub fn save_display_config(display: &DisplayConfig) {
    save_section_or_log(&get_display_config_path(), "display", display);
}

/// Loads `[desktop]` from `configs/desktop.toml`.
pub fn load_desktop_config() -> DesktopConfig {
    store::load_section(&get_desktop_config_path(), "desktop")
}

/// Persists `[desktop]` into `configs/desktop.toml`.
pub fn save_desktop_config(desktop: &DesktopConfig) {
    save_section_or_log(&get_desktop_config_path(), "desktop", desktop);
}

/// Loads `[lockscreen]` from `configs/lockscreen.toml`.
pub fn load_lockscreen_config() -> LockscreenConfig {
    store::load_section(&get_lockscreen_config_path(), "lockscreen")
}

/// Persists `[lockscreen]` into `configs/lockscreen.toml`.
pub fn save_lockscreen_config(lockscreen: &LockscreenConfig) {
    save_section_or_log(&get_lockscreen_config_path(), "lockscreen", lockscreen);
}

/// Loads `[clipboard]` from `configs/clipboard.toml`.
pub fn load_clipboard_config() -> ClipboardConfig {
    store::load_section(&get_clipboard_config_path(), "clipboard")
}

/// Persists `[clipboard]` into `configs/clipboard.toml`.
pub fn save_clipboard_config(clipboard: &ClipboardConfig) {
    save_section_or_log(&get_clipboard_config_path(), "clipboard", clipboard);
}

/// Loads `[blur]` from `configs/blur.toml`.
pub fn load_blur_config() -> BlurConfig {
    store::load_section(&get_blur_config_path(), "blur")
}

/// Persists `[blur]` into `configs/blur.toml`.
pub fn save_blur_config(blur: &BlurConfig) {
    save_section_or_log(&get_blur_config_path(), "blur", blur);
}

/// Loads `[cursor]` from `configs/cursor.toml`.
pub fn load_cursor_config() -> CursorConfig {
    store::load_section(&get_cursor_config_path(), "cursor")
}

/// Persists `[cursor]` into `configs/cursor.toml`.
pub fn save_cursor_config(cursor: &CursorConfig) {
    save_section_or_log(&get_cursor_config_path(), "cursor", cursor);
}

/// Loads `[mouse]` from `configs/input.toml`.
pub fn load_mouse_config() -> MouseConfig {
    store::load_section(&get_input_config_path(), "mouse")
}

/// Persists `[mouse]` into `configs/input.toml`.
pub fn save_mouse_config(mouse: &MouseConfig) {
    save_section_or_log(&get_input_config_path(), "mouse", mouse);
}

/// Loads `[touchpad]` from `configs/input.toml`.
pub fn load_touchpad_config() -> TouchpadConfig {
    store::load_section(&get_input_config_path(), "touchpad")
}

/// Persists `[touchpad]` into `configs/input.toml`.
pub fn save_touchpad_config(touchpad: &TouchpadConfig) {
    save_section_or_log(&get_input_config_path(), "touchpad", touchpad);
}

/// Loads `[gestures]` from `configs/input.toml`.
pub fn load_gesture_config() -> GestureConfig {
    store::load_section(&get_input_config_path(), "gestures")
}

/// Persists `[gestures]` into `configs/input.toml`.
pub fn save_gesture_config(gestures: &GestureConfig) {
    save_section_or_log(&get_input_config_path(), "gestures", gestures);
}

/// Loads `[autostart]` from the main config.
pub fn load_autostart_config() -> AutostartConfig {
    load_babydra_config().autostart
}

/// Persists `[autostart]` into the main config.
pub fn save_autostart_config(autostart: &AutostartConfig) {
    let mut config = load_babydra_config();
    config.autostart = autostart.clone();
    save_babydra_config(&config);
}

/// Loads `[power]` from the main config.
pub fn load_power_config() -> PowerConfig {
    load_babydra_config().power
}

/// Persists `[power]` into the main config.
pub fn save_power_config(power: &PowerConfig) {
    let mut config = load_babydra_config();
    config.power = power.clone();
    save_babydra_config(&config);
}

/// Loads `[wallpaper]` from the main config.
pub fn load_wallpaper_config() -> WallpaperConfig {
    load_babydra_config().wallpaper
}

/// Persists `[wallpaper]` into the main config.
pub fn save_wallpaper_config(wallpaper: &WallpaperConfig) {
    let mut config = load_babydra_config();
    config.wallpaper = wallpaper.clone();
    save_babydra_config(&config);
}

/// Loads `[notification]` from the main config.
pub fn load_notification_config() -> NotificationConfig {
    load_babydra_config().notification
}

/// Persists `[notification]` into the main config.
pub fn save_notification_config(notification: &NotificationConfig) {
    let mut config = load_babydra_config();
    config.notification = notification.clone();
    save_babydra_config(&config);
}

/// Clears every cached config document, forcing the next read to hit disk.
pub fn invalidate_cache() {
    store::invalidate_all();
}
