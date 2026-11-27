//! Lightweight, debounced persistence for desktop icon positions.
//!
//! Positions are kept in a `LazyLock<RwLock<HashMap>>` in-memory cache.
//! All mutations go into the cache instantly (zero disk I/O on the GTK thread).
//! A background debounce timer or direct flush saves to
//! `~/.babydra/configs/desktop.json`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, RwLock};

static POSITIONS: LazyLock<RwLock<HashMap<String, (i32, i32)>>> =
    LazyLock::new(|| RwLock::new(load_from_disk()));
static DIRTY: AtomicBool = AtomicBool::new(false);

fn layout_path() -> PathBuf {
    super::get_config_dir().join("desktop.json")
}

fn load_from_disk() -> HashMap<String, (i32, i32)> {
    let path = layout_path();
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Acquires the positions lock for reading, recovering from poisoning.
fn read_map() -> std::sync::RwLockReadGuard<'static, HashMap<String, (i32, i32)>> {
    match POSITIONS.read() {
        Ok(guard) => guard,
        Err(poisoned) => {
            tracing::warn!("desktop layout lock poisoned; recovering cached positions");
            poisoned.into_inner()
        }
    }
}

/// Acquires the positions lock for writing, recovering from poisoning.
fn write_map() -> std::sync::RwLockWriteGuard<'static, HashMap<String, (i32, i32)>> {
    match POSITIONS.write() {
        Ok(guard) => guard,
        Err(poisoned) => {
            tracing::warn!("desktop layout lock poisoned; recovering cached positions");
            poisoned.into_inner()
        }
    }
}

/// Returns a snapshot of all cached icon positions.
///
/// # Returns
/// A clone of the in-memory position map (no disk I/O).
pub fn get_all_positions() -> HashMap<String, (i32, i32)> {
    read_map().clone()
}

/// Reads one icon's cached position.
///
/// # Arguments
/// * `name` - Icon name to look up.
///
/// # Returns
/// The stored `(x, y)` position, or `None` if the icon has no saved position.
pub fn get_position(name: &str) -> Option<(i32, i32)> {
    read_map().get(name).copied()
}

/// Sets one icon's position in memory and marks the cache dirty for flush.
///
/// # Arguments
/// * `name` - Icon name.
/// * `x` - Horizontal position.
/// * `y` - Vertical position.
pub fn set_position(name: String, x: i32, y: i32) {
    let mut map = write_map();
    if map.get(&name) != Some(&(x, y)) {
        map.insert(name, (x, y));
        DIRTY.store(true, Ordering::Relaxed);
    }
}

/// Sets several icon positions with a single lock acquisition.
///
/// # Arguments
/// * `batch` - `(name, x, y)` triples to apply.
pub fn set_positions(batch: Vec<(String, i32, i32)>) {
    set_positions_slice(&batch);
}

/// Borrowing variant of [`set_positions`] that does not take ownership of the
/// caller's buffer.
///
/// # Arguments
/// * `batch` - `(name, x, y)` triples to apply.
pub fn set_positions_slice(batch: &[(String, i32, i32)]) {
    let mut map = write_map();
    let mut changed = false;
    for (name, x, y) in batch {
        if map.get(name) != Some(&(*x, *y)) {
            map.insert(name.clone(), (*x, *y));
            changed = true;
        }
    }
    if changed {
        DIRTY.store(true, Ordering::Relaxed);
    }
}

/// Removes cached position entries that no longer exist on disk.
///
/// # Arguments
/// * `existing_names` - Names of icons that still exist on disk.
pub fn cleanup_stale(existing_names: &[impl AsRef<str>]) {
    let mut map = write_map();
    let set: std::collections::HashSet<&str> =
        existing_names.iter().map(|name| name.as_ref()).collect();
    let initial_len = map.len();
    map.retain(|k, _| set.contains(k.as_str()));
    if map.len() != initial_len {
        DIRTY.store(true, Ordering::Relaxed);
    }
}

/// Clears all cached icon positions and marks the cache dirty for flush.
pub fn clear() {
    let mut map = write_map();
    map.clear();
    DIRTY.store(true, Ordering::Relaxed);
}

/// Flushes the cached positions to disk asynchronously when the cache is dirty.
///
/// Serialization and file errors are ignored after logging a warning, so a
/// failed flush never interrupts the caller.
pub fn flush_if_dirty() {
    if DIRTY
        .compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
    {
        let data = read_map().clone();
        let path = layout_path();

        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        std::thread::spawn(move || match serde_json::to_string_pretty(&data) {
            Ok(json) => {
                if let Err(err) = std::fs::write(&path, json) {
                    tracing::warn!("failed to save desktop layout: {err}");
                }
            }
            Err(err) => tracing::warn!("failed to serialize desktop layout: {err}"),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poisoned_positions_lock_is_recovered_for_read_and_write() {
        // Poison the shared map with a panicking writer.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = POSITIONS.write().expect("acquire positions lock");
            panic!("poison positions lock");
        }));
        assert!(POSITIONS.is_poisoned());

        // Both helpers must recover without panicking.
        write_map().insert("poison_test".to_string(), (7, 9));
        assert_eq!(get_position("poison_test"), Some((7, 9)));
        let all = get_all_positions();
        assert_eq!(all.get("poison_test"), Some(&(7, 9)));
    }
}
