use common::{KeyboardAction, ShortcutBinding, ShortcutManifest, ShortcutTarget};

use super::registry::{RegistrationError, ShortcutRegistry};
use crate::domain::keysyms::{KEY_e, KEY_F1, KEY_t};
use crate::domain::shortcuts::Mods;

const KEY_F4: u32 = KEY_F1 + 3;

fn binding(id: &str, combo: &str, target: ShortcutTarget) -> ShortcutBinding {
    ShortcutBinding {
        id: id.to_string(),
        combo: combo.to_string(),
        target,
        enabled: true,
    }
}

fn win() -> Mods {
    Mods {
        win: true,
        ..Mods::default()
    }
}

fn win_shift() -> Mods {
    Mods {
        win: true,
        shift: true,
        ..Mods::default()
    }
}

fn alt() -> Mods {
    Mods {
        alt: true,
        ..Mods::default()
    }
}

#[test]
fn registered_command_matches_only_its_exact_combo() {
    let registry = ShortcutRegistry::new();
    registry
        .register(
            "app",
            binding(
                "terminal",
                "W-S-t",
                ShortcutTarget::Command("kitty".into()),
            ),
        )
        .unwrap();

    assert_eq!(
        registry.target_for(win_shift(), KEY_t),
        Some(ShortcutTarget::Command("kitty".into()))
    );
    assert_eq!(
        registry.target_for(win(), KEY_t),
        None,
        "modifier mismatch must not match"
    );
    assert_eq!(
        registry.target_for(win_shift(), KEY_e),
        None,
        "key mismatch must not match"
    );
}

#[test]
fn register_accepts_both_command_and_action_targets() {
    let registry = ShortcutRegistry::new();
    registry
        .register(
            "app",
            binding(
                "terminal",
                "W-e",
                ShortcutTarget::Command("kitty".into()),
            ),
        )
        .unwrap();
    registry
        .register(
            "app",
            binding(
                "power",
                "A-F4",
                ShortcutTarget::Action(KeyboardAction::ToggleLauncher),
            ),
        )
        .unwrap();

    assert_eq!(
        registry.target_for(win(), KEY_e),
        Some(ShortcutTarget::Command("kitty".into()))
    );
    assert_eq!(
        registry.target_for(alt(), KEY_F4),
        Some(ShortcutTarget::Action(KeyboardAction::ToggleLauncher))
    );
}

#[test]
fn disabled_binding_never_matches() {
    let registry = ShortcutRegistry::new();
    let mut disabled = binding("terminal", "W-e", ShortcutTarget::Command("kitty".into()));
    disabled.enabled = false;
    registry.register("app", disabled).unwrap();

    assert_eq!(registry.target_for(win(), KEY_e), None);
    assert_eq!(registry.bindings().len(), 1);
}

#[test]
fn registering_same_owner_and_id_replaces_binding() {
    let registry = ShortcutRegistry::new();
    registry
        .register(
            "app",
            binding("terminal", "W-e", ShortcutTarget::Command("kitty".into())),
        )
        .unwrap();
    registry
        .register(
            "app",
            binding(
                "terminal",
                "W-e",
                ShortcutTarget::Action(KeyboardAction::Screenshot),
            ),
        )
        .unwrap();

    assert_eq!(registry.bindings().len(), 1);
    assert_eq!(
        registry.target_for(win(), KEY_e),
        Some(ShortcutTarget::Action(KeyboardAction::Screenshot))
    );
}

#[test]
fn invalid_combos_are_rejected_at_registration() {
    let registry = ShortcutRegistry::new();
    let command = || ShortcutTarget::Command("kitty".into());

    for combo in ["e", "-e", "W-", "X-e", "W-unknown"] {
        assert!(
            matches!(
                registry.register("app", binding("bad", combo, command())),
                Err(RegistrationError::InvalidCombo(_))
            ),
            "combo {combo:?} should be rejected"
        );
    }
    assert!(registry.bindings().is_empty());
}

#[test]
fn empty_command_is_rejected() {
    let registry = ShortcutRegistry::new();
    let result = registry.register(
        "app",
        binding("empty", "W-e", ShortcutTarget::Command("  ".into())),
    );
    assert!(matches!(result, Err(RegistrationError::InvalidTarget(_))));
}

#[test]
fn sync_owner_replaces_snapshot_and_skips_bad_entries() {
    let registry = ShortcutRegistry::new();
    registry
        .register(
            "app",
            binding("old", "W-e", ShortcutTarget::Command("old".into())),
        )
        .unwrap();

    let applied = registry.sync_owner(ShortcutManifest {
        owner: "app".into(),
        revision: 1,
        bindings: vec![
            binding("good", "A-F4", ShortcutTarget::Action(KeyboardAction::Screenshot)),
            binding("bad", "nonsense", ShortcutTarget::Command("x".into())),
        ],
    });

    assert_eq!(applied, 1);
    assert_eq!(registry.bindings().len(), 1);
    assert_eq!(registry.target_for(win(), KEY_e), None);
    assert_eq!(
        registry.target_for(alt(), KEY_F4),
        Some(ShortcutTarget::Action(KeyboardAction::Screenshot))
    );
}

#[test]
fn unregister_owner_and_retain_owners_remove_only_targeted_bindings() {
    let registry = ShortcutRegistry::new();
    for owner in ["topbar", "desktop"] {
        registry
            .register(
                owner,
                binding("key", "W-e", ShortcutTarget::Command(owner.into())),
            )
            .unwrap();
    }

    assert_eq!(registry.unregister_owner("topbar"), 1);
    assert_eq!(registry.bindings().len(), 1);
    assert_eq!(
        registry.target_for(win(), KEY_e),
        Some(ShortcutTarget::Command("desktop".into()))
    );

    assert_eq!(registry.retain_owners(&[]), 1);
    assert!(registry.bindings().is_empty());
}
