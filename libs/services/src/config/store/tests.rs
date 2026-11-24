use super::*;

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
struct TestSection {
    enabled: bool,
    #[serde(default)]
    name: String,
}

impl Default for TestSection {
    fn default() -> Self {
        Self {
            enabled: true,
            name: "default".into(),
        }
    }
}

#[test]
fn section_roundtrip_preserves_other_sections() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("input.toml");
    std::fs::write(&path, "[mouse]\nnatural_scroll = true\n").expect("write fixture");

    save_section(
        &path,
        "touchpad",
        &TestSection {
            enabled: false,
            name: "pad".into(),
        },
    )
    .expect("save section");

    let doc = read_doc(&path);
    assert!(doc.get("mouse").is_some());
    assert!(doc.get("touchpad").is_some());
    let touchpad: TestSection = load_section(&path, "touchpad");
    assert_eq!(touchpad.name, "pad");
    assert!(!touchpad.enabled);
}

#[test]
fn table_mut_rejects_non_table_document() {
    let mut doc = toml::Value::Integer(42);
    let err = table_mut(&mut doc).expect_err("scalar documents have no table");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn table_mut_accepts_table_document() {
    let mut doc = empty_doc();
    table_mut(&mut doc).expect("empty table is valid");
}

#[test]
fn missing_section_returns_default() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("missing.toml");
    let section: TestSection = load_section(&path, "test");
    assert_eq!(section, TestSection::default());
}

#[test]
fn migrate_sections_moves_only_legacy_values() {
    let dir = tempfile::tempdir().expect("temp dir");
    let legacy = dir.path().join("babydra.conf");
    let input = dir.path().join("input.toml");
    std::fs::write(
        &legacy,
        "[mouse]\nnatural_scroll = true\n[autostart]\ncommands = [\"a\"]\n",
    )
    .expect("write legacy");

    let changed = migrate_sections(&legacy, &[("mouse", input.as_path())]).expect("migrate");
    assert!(changed);

    let legacy_doc = read_doc(&legacy);
    assert!(legacy_doc.get("mouse").is_none());
    assert!(legacy_doc.get("autostart").is_some());

    let input_doc = read_doc(&input);
    assert_eq!(
        input_doc
            .get("mouse")
            .and_then(|mouse| mouse.get("natural_scroll"))
            .and_then(toml::Value::as_bool),
        Some(true)
    );
}

#[test]
fn target_file_wins_over_legacy_section() {
    let dir = tempfile::tempdir().expect("temp dir");
    let legacy = dir.path().join("babydra.conf");
    let input = dir.path().join("input.toml");
    std::fs::write(&legacy, "[mouse]\nnatural_scroll = true\n").expect("write legacy");
    std::fs::write(&input, "[mouse]\nnatural_scroll = false\n").expect("write target");

    let changed = migrate_sections(&legacy, &[("mouse", input.as_path())]).expect("migrate");
    assert!(changed);

    let input_doc = read_doc(&input);
    assert_eq!(
        input_doc
            .get("mouse")
            .and_then(|mouse| mouse.get("natural_scroll"))
            .and_then(toml::Value::as_bool),
        Some(false)
    );
    assert!(read_doc(&legacy).get("mouse").is_none());
}

#[test]
fn save_root_preserves_unknown_keys() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("babydra.conf");
    std::fs::write(&path, "custom = 7\n[autostart]\ncommands = []\n").expect("write fixture");

    let mut value = toml::Table::new();
    value.insert(
        "autostart".to_string(),
        toml::Value::Table(toml::Table::new()),
    );
    save_root_preserving_unknown(&path, &toml::Value::Table(value)).expect("save root");

    let doc = read_doc(&path);
    assert_eq!(doc.get("custom").and_then(toml::Value::as_integer), Some(7));
    assert!(doc.get("autostart").is_some());
}
