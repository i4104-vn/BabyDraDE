//! Directory setup and one-time migration of legacy configuration.

use std::path::{Path, PathBuf};

use serde::Serialize;

use super::paths::{
    get_babydra_dir, get_conf_path, get_config_dir, get_cursor_dir, get_keymap_path,
    get_legacy_keymap_path, get_logs_dir, get_wallpaper_dir,
};
use crate::config::store;
use crate::models::{
    BabyDraConfig, BlurConfig, ClipboardConfig, CursorConfig, DesktopConfig, DisplayConfig,
    GestureConfig, LockscreenConfig, MouseConfig, TouchpadConfig,
};

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

pub(super) fn save_root_or_log<T: Serialize>(path: &Path, value: &T, label: &str) {
    if let Err(err) = save_root(path, value) {
        eprintln!("failed to save {label} config {}: {err}", path.display());
    }
}

/// Persists a typed root document without discarding unknown keys.
///
/// Unlike [`save_root_or_log`], this returns the I/O result so callers that
/// promise persistence can propagate failures.
pub(super) fn save_root<T: Serialize>(path: &Path, value: &T) -> std::io::Result<()> {
    store::save_root_preserving_unknown(path, value)
}

pub(super) fn save_section_or_log<T: Serialize>(path: &Path, section: &str, value: &T) {
    if let Err(err) = store::save_section(path, section, value) {
        eprintln!("failed to save {} [{}]: {err}", path.display(), section);
    }
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
