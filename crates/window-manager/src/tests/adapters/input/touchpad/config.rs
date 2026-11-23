use super::*;
use smithay::reexports::input::TapButtonMap;

#[test]
fn test_touchpad_config_defaults() {
    let config = TouchpadConfig::default();
    assert!(config.tap_to_click);
    assert!(config.tap_drag);
    assert!(!config.tap_drag_lock);
    assert_eq!(config.button_map, TapButtonMap::LeftRightMiddle);
    assert!(config.dwt);
    assert!(!config.natural_scroll);
    assert_eq!(config.speed, None);
}

#[test]
fn test_touchpad_config_from_shared_config() {
    let shared = services::models::TouchpadConfig {
        tap_to_click: true,
        tap_drag: false,
        tap_drag_lock: true,
        button_map: "lmr".to_string(),
        dwt: false,
        natural_scroll: true,
        speed: Some(0.75),
    };
    let config = TouchpadConfig::from(shared);
    assert!(config.tap_to_click);
    assert!(!config.tap_drag);
    assert!(config.tap_drag_lock);
    assert_eq!(config.button_map, TapButtonMap::LeftMiddleRight);
    assert!(!config.dwt);
    assert!(config.natural_scroll);
    assert_eq!(config.speed, Some(0.75));
}

#[test]
fn test_handle_device_non_libinput() {
    struct MockDevice {
        _id: u32,
    }
    let mut mock = MockDevice { _id: 42 };
    // Should not panic on non-libinput devices (e.g. Winit devices)
    handle_device_added(&mut mock);
    handle_device_removed(&mock);
}
