//! Typed section load/save helpers.

use serde::{de::DeserializeOwned, Serialize};

use super::io::{read_doc, write_doc};
use super::*;

/// Checks whether a TOML document contains the named section.
///
/// # Arguments
/// * `path` - Path to the TOML document.
/// * `section` - Top-level section name to look for.
///
/// # Returns
/// `true` when the section is present, `false` otherwise (including missing files).
pub fn has_section(path: &Path, section: &str) -> bool {
    read_doc(path).get(section).is_some()
}

/// Loads a single section from a TOML document, falling back to `T::default()`.
///
/// Deserialization errors (missing section, malformed values) are treated as a
/// cache miss and produce the default value, keeping startup resilient to
/// user-edited config files.
///
/// # Arguments
/// * `path` - Path to the TOML document.
/// * `section` - Top-level section name to deserialize.
///
/// # Returns
/// The deserialized section, or `T::default()` when it cannot be loaded.
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
///
/// # Arguments
/// * `path` - Path to the TOML document.
///
/// # Returns
/// The deserialized document, or `T::default()` when it cannot be loaded.
pub fn load_root<T>(path: &Path) -> T
where
    T: DeserializeOwned + Default,
{
    read_doc(path).try_into().unwrap_or_default()
}

/// Inserts or replaces a section in a TOML document.
///
/// # Arguments
/// * `path` - Path to the TOML document.
/// * `section` - Top-level section name to write.
/// * `value` - Serializable value stored under `section`.
///
/// # Errors
/// Returns an error when `value` cannot be serialized to TOML, when the
/// document is not a table, or when the file cannot be written.
pub fn save_section<T>(path: &Path, section: &str, value: &T) -> std::io::Result<()>
where
    T: Serialize,
{
    let value = toml::Value::try_from(value)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
    write_section_value(path, section, value)
}

/// Inserts or replaces an already parsed section value.
///
/// # Arguments
/// * `path` - Path to the TOML document.
/// * `section` - Top-level section name to write.
/// * `value` - Parsed TOML value stored under `section`.
///
/// # Errors
/// Returns an error when the existing document is not a table or the file
/// cannot be written.
pub fn write_section_value(path: &Path, section: &str, value: toml::Value) -> std::io::Result<()> {
    let mut doc = read_doc(path);
    if !doc.is_table() {
        doc = empty_doc();
    }
    table_mut(&mut doc)?.insert(section.to_string(), value);
    write_doc(path, &doc)
}

/// Saves a typed document while keeping unknown keys from the existing file.
///
/// This is used for `babydra.conf`: users may keep custom top-level keys that
/// the typed model does not know about.
///
/// # Arguments
/// * `path` - Path to the TOML document.
/// * `value` - Serializable value whose keys replace matching top-level keys.
///
/// # Errors
/// Returns an error when `value` cannot be serialized, the document is not a
/// table, or the file cannot be written.
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
    let table = table_mut(&mut doc)?;
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
///
/// # Arguments
/// * `legacy_path` - Path to the monolithic legacy config file.
/// * `targets` - Slice of `(section, target_path)` pairs to migrate.
///
/// # Errors
/// Returns an error when the legacy document cannot be written or a target
/// section cannot be saved.
///
/// # Returns
/// `true` when at least one section was moved, `false` otherwise.
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
