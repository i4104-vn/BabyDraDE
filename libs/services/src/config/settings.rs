//! Configuration load/save logic for `~/.babydra/babydra.conf` matching babydra-core.

use crate::models::{
    BabyDraConfig, BlurConfig, DesktopConfig, GestureConfig, MouseConfig, TouchpadConfig,
};
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};
use std::time::SystemTime;

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

/// Resolves the path to `~/.babydra/babydra.conf`.
pub fn get_conf_path() -> PathBuf {
    get_babydra_dir().join("babydra.conf")
}

/// Resolves the absolute directory path to `~/.babydra/configs/`.
pub fn get_config_dir() -> PathBuf {
    get_babydra_dir().join("configs")
}

/// Resolves the path to the user's wallpaper library: `~/.babydra/wallpaper/`.
pub fn get_wallpaper_dir() -> PathBuf {
    get_babydra_dir().join("wallpaper")
}

/// Resolves the path to user logs: `~/.babydra/logs/`.
pub fn get_logs_dir() -> PathBuf {
    get_babydra_dir().join("logs")
}

/// Resolves the path to system keymap configuration: `~/.babydra/keymap.toml`.
pub fn get_keymap_path() -> PathBuf {
    get_babydra_dir().join("keymap.toml")
}

/// Ensures all standard `~/.babydra` directories and initial default files exist.
pub fn ensure_babydra_dirs() {
    let babydra_dir = get_babydra_dir();
    let configs_dir = get_config_dir();
    let wallpaper_dir = get_wallpaper_dir();
    let logs_dir = get_logs_dir();

    for dir in &[&babydra_dir, &configs_dir, &wallpaper_dir, &logs_dir] {
        let _ = std::fs::create_dir_all(dir);
    }

    // Write default wallpaper if missing
    let default_wp = wallpaper_dir.join("wallpaper.jpeg");
    if !default_wp.exists() {
        let _ = std::fs::write(&default_wp, crate::wallpaper::DEFAULT_WALLPAPER_BYTES);
    }

    // Initialize babydra.conf if missing
    let conf_path = get_conf_path();
    if !conf_path.exists() {
        let mut config = BabyDraConfig::default();
        if default_wp.exists() {
            config.wallpaper.current = default_wp.to_string_lossy().to_string();
        }
        if let Ok(content) = toml::to_string_pretty(&config) {
            let _ = std::fs::write(&conf_path, content);
        }
    }
}

type CachedConfig = Option<(Option<SystemTime>, BabyDraConfig)>;
static CONFIG_CACHE: OnceLock<RwLock<CachedConfig>> = OnceLock::new();

fn conf_mtime() -> Option<SystemTime> {
    std::fs::metadata(get_conf_path())
        .and_then(|m| m.modified())
        .ok()
}

fn load_from_disk() -> BabyDraConfig {
    let path = get_conf_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(config) = toml::from_str::<BabyDraConfig>(&content) {
                return config;
            }
        }
    }

    let mut config = BabyDraConfig::default();

    // Default wallpaper if present
    let default_wp = get_wallpaper_dir().join("wallpaper.jpeg");
    if default_wp.exists() {
        config.wallpaper.current = default_wp.to_string_lossy().to_string();
    }

    // Ensure parent directory and persist
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(content) = toml::to_string_pretty(&config) {
        let _ = std::fs::write(&path, content);
    }
    config
}

/// Loads `BabyDraConfig` from `~/.babydra/babydra.conf` (cached, re-checks mtime).
pub fn load_babydra_config() -> BabyDraConfig {
    let cache = CONFIG_CACHE.get_or_init(|| RwLock::new(None));
    {
        if let Ok(guard) = cache.read() {
            if let Some((mtime, config)) = guard.as_ref() {
                if *mtime == conf_mtime() {
                    return config.clone();
                }
            }
        }
    }

    let mtime = conf_mtime();
    let config = load_from_disk();
    if let Ok(mut guard) = cache.write() {
        *guard = Some((mtime, config.clone()));
    }
    config
}

/// Persists `BabyDraConfig` to `~/.babydra/babydra.conf`.
pub fn save_babydra_config(config: &BabyDraConfig) {
    let path = get_conf_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(content) = toml::to_string_pretty(config) {
        let _ = std::fs::write(&path, content);
    }

    if let Ok(mut guard) = CONFIG_CACHE.get_or_init(|| RwLock::new(None)).write() {
        *guard = Some((conf_mtime(), config.clone()));
    }
}

/// Invalidates the in-memory config cache to force reading from disk.
pub fn invalidate_cache() {
    if let Ok(mut guard) = CONFIG_CACHE.get_or_init(|| RwLock::new(None)).write() {
        *guard = None;
    }
}

/// Loads `DesktopConfig` from `BabyDraConfig`.
pub fn load_desktop_config() -> DesktopConfig {
    load_babydra_config().desktop
}

/// Persists `DesktopConfig` into `BabyDraConfig`.
pub fn save_desktop_config(desktop: &DesktopConfig) {
    let mut config = load_babydra_config();
    config.desktop = desktop.clone();
    save_babydra_config(&config);
}

/// Loads the shared compositor blur configuration.
pub fn load_blur_config() -> BlurConfig {
    load_babydra_config().blur
}

/// Persists the compositor blur configuration in `babydra.conf`.
pub fn save_blur_config(blur: &BlurConfig) {
    let mut config = load_babydra_config();
    config.blur = blur.clone();
    save_babydra_config(&config);
}

/// Loads the touchpad / trackpad configuration from `~/.babydra/babydra.conf`.
///
/// Reads the `[touchpad]` section from the cached or on-disk configuration,
/// falling back to defaults if not yet defined.
///
/// # Returns
///
/// Returns the current [`TouchpadConfig`] settings.
pub fn load_touchpad_config() -> TouchpadConfig {
    load_babydra_config().touchpad
}

/// Persists the touchpad / trackpad configuration into `~/.babydra/babydra.conf`.
///
/// Updates the `[touchpad]` section and flushes changes to disk.
///
/// # Arguments
///
/// * `touchpad` - Reference to the updated [`TouchpadConfig`] to persist.
pub fn save_touchpad_config(touchpad: &TouchpadConfig) {
    let mut config = load_babydra_config();
    config.touchpad = touchpad.clone();
    save_babydra_config(&config);
}

/// Loads gesture configuration from `~/.babydra/babydra.conf`.
///
/// Reads the `[gestures]` section from the cached or on-disk configuration,
/// falling back to defaults if the section is absent.
///
/// # Returns
///
/// Returns the current [`GestureConfig`] settings.
pub fn load_gesture_config() -> GestureConfig {
    load_babydra_config().gestures
}

/// Persists gesture configuration into `~/.babydra/babydra.conf`.
///
/// Updates the `[gestures]` section and flushes the full config to disk.
///
/// # Arguments
///
/// * `gestures` - Reference to the updated [`GestureConfig`] to persist.
pub fn save_gesture_config(gestures: &GestureConfig) {
    let mut config = load_babydra_config();
    config.gestures = gestures.clone();
    save_babydra_config(&config);
}

/// Loads mouse / pointer configuration from `~/.babydra/babydra.conf`.
///
/// Reads the `[mouse]` section from the cached or on-disk configuration,
/// falling back to defaults if the section is absent.
///
/// # Returns
///
/// Returns the current [`MouseConfig`] settings.
pub fn load_mouse_config() -> MouseConfig {
    load_babydra_config().mouse
}

/// Persists mouse / pointer configuration into `~/.babydra/babydra.conf`.
///
/// Updates the `[mouse]` section and flushes the full config to disk.
///
/// # Arguments
///
/// * `mouse` - Reference to the updated [`MouseConfig`] to persist.
pub fn save_mouse_config(mouse: &MouseConfig) {
    let mut config = load_babydra_config();
    config.mouse = mouse.clone();
    save_babydra_config(&config);
}
