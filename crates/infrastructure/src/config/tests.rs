//! Tests for .babydra configuration and settings.

use super::*;

#[test]
fn test_babydra_paths() {
    let babydra_dir = get_babydra_dir();
    assert!(babydra_dir.ends_with(".babydra"));

    let conf_path = get_conf_path();
    assert_eq!(conf_path, babydra_dir.join("babydra.conf"));

    let config_dir = get_config_dir();
    assert_eq!(config_dir, babydra_dir.join("configs"));

    let wp_dir = get_wallpaper_dir();
    assert_eq!(wp_dir, babydra_dir.join("wallpaper"));

    let logs_dir = get_logs_dir();
    assert_eq!(logs_dir, babydra_dir.join("logs"));
}

#[test]
fn test_babydra_config_toml_roundtrip() {
    let config = BabyDraConfig {
        first_login: true,
        power: PowerConfig {
            profile: "performance".to_string(),
            ..Default::default()
        },
        wallpaper: WallpaperConfig {
            current: "/home/user/.babydra/wallpaper/test.svg".to_string(),
            mode: "static".to_string(),
        },
        desktop: DesktopConfig {
            icon_size: 64,
            ..Default::default()
        },
        ..Default::default()
    };

    let toml_str = toml::to_string_pretty(&config).expect("failed to serialize to TOML");
    assert!(toml_str.contains("1st = true"));
    assert!(toml_str.contains("performance"));
    assert!(toml_str.contains("test.svg"));

    let deserialized: BabyDraConfig =
        toml::from_str(&toml_str).expect("failed to deserialize TOML");
    assert!(deserialized.first_login);
    assert_eq!(deserialized.power.profile, "performance");
    assert_eq!(deserialized.wallpaper.current, config.wallpaper.current);
    assert_eq!(deserialized.desktop.icon_size, 64);
}

#[test]
fn test_desktop_layout_cache() {
    desktop_layout::clear();
    assert_eq!(desktop_layout::get_all_positions().len(), 0);

    desktop_layout::set_position("test_folder".to_string(), 120, 240);
    assert_eq!(
        desktop_layout::get_position("test_folder"),
        Some((120, 240))
    );

    let batch = vec![
        ("item1".to_string(), 10, 20),
        ("item2".to_string(), 30, 40),
    ];
    desktop_layout::set_positions(batch);
    assert_eq!(desktop_layout::get_position("item1"), Some((10, 20)));
    assert_eq!(desktop_layout::get_position("item2"), Some((30, 40)));

    let existing = vec!["item1".to_string(), "item2".to_string()];
    desktop_layout::cleanup_stale(&existing);
    assert_eq!(desktop_layout::get_position("test_folder"), None);
    assert_eq!(desktop_layout::get_position("item1"), Some((10, 20)));

    desktop_layout::clear();
    assert_eq!(desktop_layout::get_all_positions().len(), 0);
}
