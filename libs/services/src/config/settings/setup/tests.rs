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
