//! Generic TOML document store with mtime-based caching and atomic writes.
//!
//! Configuration is split across `~/.babydra/babydra.conf` (main settings)
//! and `~/.babydra/configs/*.toml` (subsystem settings). This module keeps a
//! parsed [`toml::Value`] document per path so callers can load or update a
//! single section without touching the rest of the file.

use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
struct CacheEntry {
    mtime: Option<SystemTime>,
    doc: toml::Value,
}

static CACHE: OnceLock<RwLock<HashMap<PathBuf, CacheEntry>>> = OnceLock::new();

fn cache() -> &'static RwLock<HashMap<PathBuf, CacheEntry>> {
    CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn file_mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

fn empty_doc() -> toml::Value {
    toml::Value::Table(toml::Table::new())
}

/// Loads a TOML document, using the cached copy while its mtime is unchanged.
pub fn read_doc(path: &Path) -> toml::Value {
    let mtime = file_mtime(path);
    if let Ok(guard) = cache().read() {
        if let Some(entry) = guard.get(path) {
            if entry.mtime == mtime {
                return entry.doc.clone();
            }
        }
    }

    let doc = std::fs::read_to_string(path)
        .ok()
        .and_then(|content| toml::from_str::<toml::Value>(&content).ok())
        .unwrap_or_else(empty_doc);

    if let Ok(mut guard) = cache().write() {
        guard.insert(
            path.to_path_buf(),
            CacheEntry {
                mtime,
                doc: doc.clone(),
            },
        );
    }
    doc
}

/// Writes a TOML document atomically and refreshes the cache entry.
pub fn write_doc(path: &Path, doc: &toml::Value) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = toml::to_string_pretty(doc)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| "config".into());
    let tmp = path.with_file_name(format!(".{file_name}.tmp-{}-{nonce}", std::process::id()));

    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, path)?;

    if let Ok(mut guard) = cache().write() {
        guard.insert(
            path.to_path_buf(),
            CacheEntry {
                mtime: file_mtime(path),
                doc: doc.clone(),
            },
        );
    }
    Ok(())
}

/// Returns `true` when the document contains `section`.
pub fn has_section(path: &Path, section: &str) -> bool {
    read_doc(path).get(section).is_some()
}

/// Loads a single section from a TOML document, falling back to `T::default()`.
pub fn load_section<T>(path: &Path, section: &str) -> T
where
    T: DeserializeOwned + Default,
{
    read_doc(path)
        .get(section)
        .cloned()
        .and_then(|value| value.try_into().ok())
        .unwrap_or_default()
}

/// Loads a whole TOML document as a typed value, falling back to `T::default()`.
pub fn load_root<T>(path: &Path) -> T
where
    T: DeserializeOwned + Default,
{
    read_doc(path).try_into().unwrap_or_default()
}

/// Inserts or replaces a section in a TOML document.
pub fn save_section<T>(path: &Path, section: &str, value: &T) -> std::io::Result<()>
where
    T: Serialize,
{
    let value = toml::Value::try_from(value)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
    write_section_value(path, section, value)
}

/// Inserts or replaces an already parsed section value.
pub fn write_section_value(path: &Path, section: &str, value: toml::Value) -> std::io::Result<()> {
    let mut doc = read_doc(path);
    if !doc.is_table() {
        doc = empty_doc();
    }
    doc.as_table_mut()
        .expect("TOML document is always a table")
        .insert(section.to_string(), value);
    write_doc(path, &doc)
}

/// Saves a typed document while keeping unknown keys from the existing file.
///
/// This is used for `babydra.conf`: users may keep custom top-level keys that
/// the typed model does not know about.
pub fn save_root_preserving_unknown<T>(path: &Path, value: &T) -> std::io::Result<()>
where
    T: Serialize,
{
    let new_doc = toml::Value::try_from(value)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
    let Some(new_table) = new_doc.as_table() else {
        return write_doc(path, &new_doc);
    };

    let mut doc = read_doc(path);
    if !doc.is_table() {
        doc = empty_doc();
    }
    let table = doc.as_table_mut().expect("TOML document is always a table");
    for (key, value) in new_table {
        table.insert(key.clone(), value.clone());
    }
    write_doc(path, &doc)
}

/// Moves legacy sections out of a monolithic config file.
///
/// For every `(section, target_path)` pair, the section is removed from
/// `legacy_path` and written to `target_path` when the target does not already
/// contain that section. A target file always wins over the legacy value.
pub fn migrate_sections(legacy_path: &Path, targets: &[(&str, &Path)]) -> std::io::Result<bool> {
    let mut legacy = read_doc(legacy_path);
    let Some(table) = legacy.as_table_mut() else {
        return Ok(false);
    };

    let mut changed = false;
    for (section, target) in targets {
        if let Some(value) = table.remove(*section) {
            changed = true;
            if !has_section(target, section) {
                write_section_value(target, section, value)?;
            }
        }
    }

    if changed {
        write_doc(legacy_path, &legacy)?;
    }
    Ok(changed)
}

/// Clears every cached document, forcing the next read to hit disk.
pub fn invalidate_all() {
    if let Ok(mut guard) = cache().write() {
        guard.clear();
    }
}

#[cfg(test)]
mod tests {
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
}
