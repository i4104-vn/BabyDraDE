//! Configuration paths, loaders and savers for the split BabyDraDE config.
//!
//! Main settings are kept in `~/.babydra/babydra.conf`. Subsystem settings are
//! split into small files under `~/.babydra/configs/` (see [`settings`]).

/// Cached icon positions with debounced writes to `configs/desktop.json`.
pub mod desktop_layout;
/// Serializable configuration models.
pub mod models;
/// Settings file paths and typed loaders.
pub mod settings;
mod store;

pub use settings::{
    ensure_babydra_dirs, get_babydra_dir, get_blur_config_path, get_clipboard_config_path,
    get_conf_path, get_config_dir, get_cursor_config_path, get_cursor_dir, get_desktop_config_path,
    get_display_config_path, get_home_dir, get_input_config_path, get_keymap_path,
    get_legacy_keymap_path, get_lockscreen_config_path, get_logs_dir, get_wallpaper_dir,
    invalidate_cache, load_autostart_config, load_babydra_config, load_blur_config,
    load_clipboard_config, load_cursor_config, load_desktop_config, load_display_config,
    load_display_config_from, load_gesture_config, load_lockscreen_config, load_mouse_config,
    load_notification_config, load_power_config, load_touchpad_config, load_wallpaper_config,
    save_autostart_config, save_babydra_config, save_blur_config, save_clipboard_config,
    save_cursor_config, save_desktop_config, save_display_config, save_gesture_config,
    save_lockscreen_config, save_mouse_config, save_notification_config, save_power_config,
    save_touchpad_config, save_wallpaper_config,
};
