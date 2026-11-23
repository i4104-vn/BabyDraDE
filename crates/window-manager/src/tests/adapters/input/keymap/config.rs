use super::*;

#[test]
fn splits_combos() {
    assert_eq!(
        split_combo("W-S-t"),
        Some(("W-S".to_string(), "t".to_string()))
    );
    assert_eq!(split_combo("t"), None);
}

#[test]
fn parses_a_real_config() {
    let raw = r#"
[shortcuts]
"W-A-e" = "emacs"
"W-S-t" = "kitty"
"bad" = "ignored"
"W-X" = ""
"#;
    let config: ConfigFile = toml::from_str(raw).unwrap();
    assert_eq!(config.shortcuts.len(), 4);

    let mut shortcuts = Vec::new();
    for (combo, command) in config.shortcuts {
        if let Some((modifiers, key)) = split_combo(&combo) {
            if !command.trim().is_empty() {
                shortcuts.push(Shortcut {
                    id: shortcuts.len() + 1,
                    modifiers,
                    key,
                    command,
                    enabled: true,
                });
            }
        }
    }
    assert_eq!(shortcuts.len(), 2);
    let combos: Vec<_> = shortcuts
        .iter()
        .map(|s| (s.modifiers.as_str(), s.key.as_str()))
        .collect();
    assert!(combos.contains(&("W-A", "e")));
    assert!(combos.contains(&("W-S", "t")));
}
