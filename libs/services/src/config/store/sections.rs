//! Typed section load/save helpers.

use serde::{de::DeserializeOwned, Serialize};

use super::io::{read_doc, write_doc};
use super::*;

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
