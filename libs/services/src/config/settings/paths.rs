//! Filesystem paths for the split BabyDraDE configuration.

use std::path::PathBuf;

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
