use super::*;

#[test]
fn test_mouse_config_defaults() {
    let config = MouseConfig::default();
    assert!(!config.natural_scroll);
    assert_eq!(config.speed, None);
    assert!(!config.middle_emulation);
}

#[test]
fn test_mouse_config_from_shared_config() {
    let shared = services::models::MouseConfig {
        natural_scroll: true,
        speed: Some(0.8),
        middle_emulation: true,
    };
    let config = MouseConfig::from(shared);
    assert!(config.natural_scroll);
    assert_eq!(config.speed, Some(0.8));
    assert!(config.middle_emulation);
}

#[test]
fn test_mouse_config_speed_clamping() {
    let shared = services::models::MouseConfig {
        natural_scroll: false,
        speed: Some(5.0),
        middle_emulation: false,
    };
    let config = MouseConfig::from(shared);
    assert_eq!(config.speed, Some(1.0));

    let shared_neg = services::models::MouseConfig {
        natural_scroll: false,
        speed: Some(-5.0),
        middle_emulation: false,
    };
    let config_neg = MouseConfig::from(shared_neg);
    assert_eq!(config_neg.speed, Some(-1.0));
}
