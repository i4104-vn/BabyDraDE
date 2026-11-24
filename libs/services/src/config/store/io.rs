//! Cached TOML document I/O.

use super::*;

pub(crate) fn read_doc(path: &Path) -> toml::Value {
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
pub(crate) fn write_doc(path: &Path, doc: &toml::Value) -> std::io::Result<()> {
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

/// Clears the cached TOML documents.
pub(crate) fn invalidate_all() {
    if let Ok(mut guard) = cache().write() {
        guard.clear();
    }
}
