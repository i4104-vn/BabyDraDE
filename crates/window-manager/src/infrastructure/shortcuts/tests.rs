use common::{KeyboardAction, ShortcutBinding, ShortcutManifest, ShortcutTarget};

use super::handler::apply;
use crate::application::shortcuts::ShortcutRegistry;
use crate::domain::keysyms::{KEY_e, KEY_F1};
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

fn manifest(owner: &str, bindings: Vec<ShortcutBinding>) -> ShortcutManifest {
    ShortcutManifest {
        owner: owner.to_string(),
        revision: 1,
        bindings,
    }
}

#[test]
fn apply_installs_published_bindings() {
    let registry = ShortcutRegistry::new();
    apply(
        &registry,
        &[manifest(
            "topbar",
            vec![binding(
                "clipboard",
                "W-e",
                ShortcutTarget::Action(KeyboardAction::ToggleLauncher),
            )],
        )],
    );

    let mods = Mods {
        win: true,
        ..Mods::default()
    };
    assert!(registry.target_for(mods, KEY_e).is_some());
}

#[test]
fn apply_removes_bindings_of_apps_that_stopped_publishing() {
    let registry = ShortcutRegistry::new();
    let first = manifest(
        "topbar",
        vec![binding(
            "key",
            "A-F4",
            ShortcutTarget::Action(KeyboardAction::Screenshot),
        )],
    );
    apply(&registry, std::slice::from_ref(&first));

    let mods = Mods {
        alt: true,
        ..Mods::default()
    };
    assert!(registry.target_for(mods, KEY_F4).is_some());

    apply(&registry, &[]);
    assert_eq!(registry.bindings().len(), 0);
}

#[test]
fn apply_replaces_an_apps_previous_snapshot() {
    let registry = ShortcutRegistry::new();
    let mods = Mods {
        alt: true,
        ..Mods::default()
    };
    apply(
        &registry,
        &[manifest(
            "topbar",
            vec![binding(
                "old",
                "A-F4",
                ShortcutTarget::Action(KeyboardAction::Screenshot),
            )],
        )],
    );

    apply(
        &registry,
        &[manifest(
            "topbar",
            vec![binding(
                "new",
                "A-F4",
                ShortcutTarget::Action(KeyboardAction::ToggleLauncher),
            )],
        )],
    );

    assert_eq!(registry.bindings().len(), 1);
    assert_eq!(
        registry.target_for(mods, KEY_F4),
        Some(ShortcutTarget::Action(KeyboardAction::ToggleLauncher))
    );
}
