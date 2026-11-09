//! Configuration path resolvers and settings models matching babydra-core.

pub mod desktop_layout;
pub mod models;
pub mod settings;

pub use models::{
    BabyDraConfig, ClipboardConfig, CustomContextItem, DesktopConfig, DisplayConfig,
    DisplayMonitorSetting, LockscreenConfig, NotificationConfig, PowerConfig, WallpaperConfig,
};

pub use settings::{
    ensure_babydra_dirs, get_babydra_dir, get_conf_path, get_config_dir, get_home_dir,
    get_keymap_path, get_logs_dir, get_wallpaper_dir, invalidate_cache, load_babydra_config,
    load_desktop_config, save_babydra_config, save_desktop_config,
};

#[cfg(test)]
mod tests;
