use super::*;
use super::handler::key_name_to_keysym;
use super::models::ResolvedShortcut;
use crate::domain::keysyms;

#[test]
fn maps_letters_to_lowercase_keysyms() {
    assert_eq!(key_name_to_keysym("s"), Some(keysyms::KEY_s));
    assert_eq!(key_name_to_keysym("A"), Some(keysyms::KEY_a));
    assert_eq!(key_name_to_keysym("f12"), Some(keysyms::KEY_F12));
    assert_eq!(key_name_to_keysym("Tab"), Some(keysyms::KEY_Tab));
    assert_eq!(key_name_to_keysym("nope"), None);
}

#[test]
fn resolves_modifiers_and_key() {
    let shortcut = Shortcut {
        id: 1,
        modifiers: "W-S".into(),
        key: "s".into(),
        command: "echo hi".into(),
        enabled: true,
    };
    let resolved = ResolvedShortcut::resolve(&shortcut).unwrap();
    assert_eq!(resolved.keysym, keysyms::KEY_s);
    assert_eq!(
        resolved.mods,
        Mods {
            shift: true,
            win: true,
            ..Mods::default()
        }
    );
}

#[test]
fn matches_exact_modifiers_and_keysym() {
    let shortcuts = vec![Shortcut {
        id: 1,
        modifiers: "W-A".into(),
        key: "e".into(),
        command: "emacs".into(),
        enabled: true,
    }];
    let mods = Mods {
        win: true,
        alt: true,
        ..Mods::default()
    };
    assert_eq!(
        command_for(mods, keysyms::KEY_e, &shortcuts).as_deref(),
        Some("emacs")
    );
    assert_eq!(command_for(mods, keysyms::KEY_t, &shortcuts), None);
}
