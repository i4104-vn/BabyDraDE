//! Paths and typed load/save helpers for the split BabyDraDE configuration.
//!
//! Main settings live in `~/.babydra/babydra.conf`:
//! - `1st`
//! - `[autostart]`, `[power]`, `[wallpaper]`, `[notification]`
//! - any unknown top-level keys the user keeps there
//!
//! Subsystem settings live in small files under `~/.babydra/configs/`:
//! - `display.toml`    `[display]`
//! - `desktop.toml`    `[desktop]`
//! - `lockscreen.toml` `[lockscreen]`
//! - `clipboard.toml`  `[clipboard]`
//! - `blur.toml`       `[blur]`
//! - `cursor.toml`     `[cursor]`
//! - `input.toml`      `[mouse]`, `[touchpad]`, `[gestures]`
//! - `keymap.toml`     `[shortcuts]`

use crate::models::{
    AutostartConfig, BabyDraConfig, BlurConfig, ClipboardConfig, CursorConfig, DesktopConfig,
    DisplayConfig, GestureConfig, LockscreenConfig, MouseConfig, NotificationConfig, PowerConfig,
    TouchpadConfig, WallpaperConfig,
};
use serde::Serialize;
use std::path::{Path, PathBuf};

use super::store;

/// Resolves the current user's home directory.
pub fn get_home_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return PathBuf::from(home);
        }
    }
    gio::glib::home_dir()
}

/// Resolves the root `.babydra` directory: `~/.babydra`.
pub fn get_babydra_dir() -> PathBuf {
    get_home_dir().join(".babydra")
}

/// Resolves the path to the main config `~/.babydra/babydra.conf`.
pub fn get_conf_path() -> PathBuf {
    get_babydra_dir().join("babydra.conf")
}

/// Resolves the directory for split subsystem configs: `~/.babydra/configs/`.
pub fn get_config_dir() -> PathBuf {
    get_babydra_dir().join("configs")
}

/// Resolves `~/.babydra/configs/display.toml`.
pub fn get_display_config_path() -> PathBuf {
    get_config_dir().join("display.toml")
}

/// Resolves `~/.babydra/configs/desktop.toml`.
pub fn get_desktop_config_path() -> PathBuf {
    get_config_dir().join("desktop.toml")
}

/// Resolves `~/.babydra/configs/lockscreen.toml`.
pub fn get_lockscreen_config_path() -> PathBuf {
    get_config_dir().join("lockscreen.toml")
}

/// Resolves `~/.babydra/configs/clipboard.toml`.
pub fn get_clipboard_config_path() -> PathBuf {
    get_config_dir().join("clipboard.toml")
}

/// Resolves `~/.babydra/configs/blur.toml`.
pub fn get_blur_config_path() -> PathBuf {
    get_config_dir().join("blur.toml")
}

/// Resolves `~/.babydra/configs/cursor.toml`.
pub fn get_cursor_config_path() -> PathBuf {
    get_config_dir().join("cursor.toml")
}

/// Resolves `~/.babydra/configs/input.toml` (`[mouse]`, `[touchpad]`, `[gestures]`).
pub fn get_input_config_path() -> PathBuf {
    get_config_dir().join("input.toml")
}

/// Resolves the wallpaper directory: `~/.babydra/wallpaper/`.
pub fn get_wallpaper_dir() -> PathBuf {
    get_babydra_dir().join("wallpaper")
}

/// Resolves the log directory: `~/.babydra/logs/`.
pub fn get_logs_dir() -> PathBuf {
    get_babydra_dir().join("logs")
}

/// Resolves the cursor theme directory: `~/.babydra/cursor/`.
pub fn get_cursor_dir() -> PathBuf {
    get_babydra_dir().join("cursor")
}

/// Resolves the keymap config: `~/.babydra/configs/keymap.toml`.
pub fn get_keymap_path() -> PathBuf {
    get_config_dir().join("keymap.toml")
}

/// Resolves the legacy keymap location: `~/.babydra/keymap.toml`.
pub fn get_legacy_keymap_path() -> PathBuf {
    get_babydra_dir().join("keymap.toml")
}

/// Creates the standard `~/.babydra` directories and initializes default
/// config files. Legacy sections in `babydra.conf` are migrated once.
pub fn ensure_babydra_dirs() {
    let babydra_dir = get_babydra_dir();
    let configs_dir = get_config_dir();
    let wallpaper_dir = get_wallpaper_dir();
    let logs_dir = get_logs_dir();
    let cursor_dir = get_cursor_dir();

    for dir in &[
        &babydra_dir,
        &configs_dir,
        &wallpaper_dir,
        &logs_dir,
        &cursor_dir,
    ] {
        if let Err(err) = std::fs::create_dir_all(dir) {
            eprintln!("failed to create {}: {err}", dir.display());
        }
    }

    let default_wp = wallpaper_dir.join("wallpaper.jpeg");
    if !default_wp.exists() {
        if let Err(err) = std::fs::write(&default_wp, crate::wallpaper::DEFAULT_WALLPAPER_BYTES) {
            eprintln!("failed to write default wallpaper: {err}");
        }
    }

    migrate_legacy_config();
    migrate_legacy_keymap();
    ensure_main_config(&default_wp);
    ensure_subsystem_files();
}

fn migration_targets(config_dir: &Path) -> Vec<(&'static str, PathBuf)> {
    vec![
        ("display", config_dir.join("display.toml")),
        ("desktop", config_dir.join("desktop.toml")),
        ("lockscreen", config_dir.join("lockscreen.toml")),
        ("clipboard", config_dir.join("clipboard.toml")),
        ("blur", config_dir.join("blur.toml")),
        ("cursor", config_dir.join("cursor.toml")),
        ("mouse", config_dir.join("input.toml")),
        ("touchpad", config_dir.join("input.toml")),
        ("gestures", config_dir.join("input.toml")),
    ]
}

fn migrate_legacy_config_at(legacy_path: &Path, config_dir: &Path) -> std::io::Result<bool> {
    let targets = migration_targets(config_dir);
    let refs: Vec<(&str, &Path)> = targets
        .iter()
        .map(|(section, path)| (*section, path.as_path()))
        .collect();
    store::migrate_sections(legacy_path, &refs)
}

fn migrate_legacy_config() {
    if let Err(err) = migrate_legacy_config_at(&get_conf_path(), &get_config_dir()) {
        eprintln!("failed to migrate legacy BabyDra config: {err}");
    }
}

fn migrate_legacy_keymap() {
    let target = get_keymap_path();
    let legacy = get_legacy_keymap_path();
    if target.exists() || !legacy.exists() {
        return;
    }

    if let Some(parent) = target.parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            eprintln!("failed to create keymap directory: {err}");
            return;
        }
    }
    if let Err(err) = std::fs::copy(&legacy, &target) {
        eprintln!("failed to migrate keymap config: {err}");
    }
}

fn ensure_main_config(default_wallpaper: &Path) {
    let path = get_conf_path();
    if path.exists() {
        return;
    }

    let mut config = BabyDraConfig::default();
    if default_wallpaper.exists() {
        config.wallpaper.current = default_wallpaper.to_string_lossy().to_string();
    }
    save_root_or_log(&path, &config, "main");
}

fn ensure_subsystem_files() {
    ensure_subsystem_files_at(&get_config_dir());
}

fn ensure_subsystem_files_at(config_dir: &Path) {
    ensure_section(
        &config_dir.join("display.toml"),
        "display",
        &DisplayConfig::default(),
    );
    ensure_section(
        &config_dir.join("desktop.toml"),
        "desktop",
        &DesktopConfig::default(),
    );
    ensure_section(
        &config_dir.join("lockscreen.toml"),
        "lockscreen",
        &LockscreenConfig::default(),
    );
    ensure_section(
        &config_dir.join("clipboard.toml"),
        "clipboard",
        &ClipboardConfig::default(),
    );
    ensure_section(
        &config_dir.join("blur.toml"),
        "blur",
        &BlurConfig::default(),
    );
    ensure_section(
        &config_dir.join("cursor.toml"),
        "cursor",
        &CursorConfig::default(),
    );

    let input = config_dir.join("input.toml");
    ensure_section(&input, "mouse", &MouseConfig::default());
    ensure_section(&input, "touchpad", &TouchpadConfig::default());
    ensure_section(&input, "gestures", &GestureConfig::default());
}

fn ensure_section<T: Serialize>(path: &Path, section: &str, default: &T) {
    if store::has_section(path, section) {
        return;
    }
    if let Err(err) = store::save_section(path, section, default) {
        eprintln!(
            "failed to initialize {} [{}]: {err}",
            path.display(),
            section
        );
    }
}

fn save_root_or_log<T: Serialize>(path: &Path, value: &T, label: &str) {
    if let Err(err) = store::save_root_preserving_unknown(path, value) {
        eprintln!("failed to save {label} config {}: {err}", path.display());
    }
}

fn save_section_or_log<T: Serialize>(path: &Path, section: &str, value: &T) {
    if let Err(err) = store::save_section(path, section, value) {
        eprintln!("failed to save {} [{}]: {err}", path.display(), section);
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_uses_expected_subsystem_files() {
        let dir = tempfile::tempdir().expect("temp dir");
        let legacy = dir.path().join("babydra.conf");
        let configs = dir.path().join("configs");
        std::fs::create_dir_all(&configs).expect("create configs");
        std::fs::write(
            &legacy,
            "[blur]\nradius = 22.0\n[mouse]\nnatural_scroll = true\n[autostart]\ncommands = [\"a\"]\n",
        )
        .expect("write legacy");

        assert!(migrate_legacy_config_at(&legacy, &configs).expect("migrate"));

        let legacy_doc = store::read_doc(&legacy);
        assert!(legacy_doc.get("blur").is_none());
        assert!(legacy_doc.get("mouse").is_none());
        assert!(legacy_doc.get("autostart").is_some());

        let blur_doc = store::read_doc(&configs.join("blur.toml"));
        assert!(blur_doc.get("blur").is_some());
        let input_doc = store::read_doc(&configs.join("input.toml"));
        assert!(input_doc.get("mouse").is_some());
    }

    #[test]
    fn ensure_creates_all_subsystem_files() {
        let dir = tempfile::tempdir().expect("temp dir");
        let configs = dir.path().join("configs");
        std::fs::create_dir_all(&configs).expect("create configs");

        ensure_subsystem_files_at(&configs);

        for name in [
            "blur.toml",
            "clipboard.toml",
            "cursor.toml",
            "desktop.toml",
            "display.toml",
            "input.toml",
            "lockscreen.toml",
        ] {
            assert!(configs.join(name).exists(), "missing {name}");
        }

        let input_doc = store::read_doc(&configs.join("input.toml"));
        assert!(input_doc.get("mouse").is_some());
        assert!(input_doc.get("touchpad").is_some());
        assert!(input_doc.get("gestures").is_some());
    }
}
