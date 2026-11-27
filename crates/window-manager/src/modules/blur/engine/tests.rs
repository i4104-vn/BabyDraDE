use super::*;

#[test]
fn test_blur_engine_rules_and_matching() {
    let mut engine = BlurEngine::new();
    engine.params.blur_all = false;
    engine.params.app_rules = vec![
        "foot".to_string(),
        "alacritty".to_string(),
        "kitty".to_string(),
        "babydra-panel".to_string(),
    ];

    // Direct matching
    assert!(engine.matches_app_or_title(Some("foot"), None));
    assert!(engine.matches_app_or_title(Some("alacritty"), None));
    assert!(engine.matches_app_or_title(Some("Kitty"), None)); // case insensitive
    assert!(engine.matches_app_or_title(None, Some("foot terminal")));
    assert!(!engine.matches_app_or_title(Some("unknown-app"), Some("game")));

    // Add custom app
    engine.add_app("my-custom-terminal");
    assert!(engine.matches_app_or_title(Some("my-custom-terminal"), None));

    // Remove app
    engine.remove_app("my-custom-terminal");
    assert!(!engine.matches_app_or_title(Some("my-custom-terminal"), None));

    // Toggle app
    let toggled_on = engine.toggle_app("my-custom-terminal");
    assert!(toggled_on);
    assert!(engine.matches_app_or_title(Some("my-custom-terminal"), None));

    let toggled_off = engine.toggle_app("my-custom-terminal");
    assert!(!toggled_off);
    assert!(!engine.matches_app_or_title(Some("my-custom-terminal"), None));

    // Blur all toggle
    engine.set_blur_all(true);
    assert!(!engine.matches_layer("random-layer"));
    assert!(!engine.matches_layer("babydra-topbar"));
    engine.set_blur_all(false);
    assert!(engine.matches_layer("babydra-panel"));
    assert!(!engine.matches_layer("babydra-topbar"));
    assert!(!engine.matches_layer("random-layer"));
}

#[test]
fn blur_all_matches_every_app() {
    let mut engine = BlurEngine::new();
    engine.params.blur_all = true;
    assert!(engine.matches_app_or_title(Some("unknown-app"), Some("anything")));
    assert!(engine.matches_app_or_title(None, None));

    engine.params.blur_all = false;
    assert!(!engine.matches_app_or_title(Some("unknown-app"), Some("anything")));
}
