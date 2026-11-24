//! Generic TOML document store with mtime-based caching and atomic writes.
//!
//! Configuration is split across `~/.babydra/babydra.conf` (main settings)
//! and `~/.babydra/configs/*.toml` (subsystem settings). This module keeps a
//! parsed [`toml::Value`] document per path so callers can load or update a
//! single section without touching the rest of the file.

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

/// Returns the document's table, reporting invalid documents instead of panicking.
fn table_mut(doc: &mut toml::Value) -> std::io::Result<&mut toml::Table> {
    doc.as_table_mut().ok_or_else(|| {
        tracing::warn!("configuration document is not a TOML table");
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "TOML document is not a table",
        )
    })
}

mod io;
mod sections;

#[cfg(test)]
mod tests;

pub(crate) use io::invalidate_all;
#[cfg(test)]
pub(crate) use io::read_doc;
pub use sections::{
    has_section, load_root, load_section, migrate_sections, save_root_preserving_unknown,
    save_section,
};
