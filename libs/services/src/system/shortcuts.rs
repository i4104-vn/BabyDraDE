//! File-based shortcut registration API for BabyDraDE apps.
//!
//! Every app publishes one manifest under `$XDG_RUNTIME_DIR/babydra-shortcuts`.
//! The compositor watches that directory and applies the bindings to its
//! keyboard pipeline. One file per owner keeps concurrent apps from
//! overwriting each other, and an atomic rename keeps readers from ever
//! seeing a half-written manifest.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

pub use common::{ShortcutBinding, ShortcutManifest, ShortcutTarget};

/// Directory name reserved for shortcut manifests inside the runtime dir.
const SHORTCUT_DIR: &str = "babydra-shortcuts";
/// File extension used for every manifest.
const MANIFEST_EXT: &str = "json";

/// Failure returned by a shortcut publication operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutError {
    /// The supplied owner id was empty or whitespace only.
    InvalidOwner,
    /// The manifest could not be read or written.
    Io(String),
    /// An existing manifest was present but could not be decoded.
    InvalidManifest(String),
}

impl fmt::Display for ShortcutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOwner => f.write_str("shortcut owner must not be empty"),
            Self::Io(message) => write!(f, "shortcut I/O error: {message}"),
            Self::InvalidManifest(message) => write!(f, "invalid shortcut manifest: {message}"),
        }
    }
}

impl std::error::Error for ShortcutError {}

/// Returns the directory holding all published shortcut manifests.
///
/// # Returns
///
/// `$XDG_RUNTIME_DIR/babydra-shortcuts`, or its `/tmp` fallback when no usable
/// runtime directory is available.
pub fn shortcut_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        let path = PathBuf::from(dir);
        if path.is_dir() {
            return path.join(SHORTCUT_DIR);
        }
    }
    PathBuf::from("/tmp").join(SHORTCUT_DIR)
}

/// Reads every published manifest, ordered by owner for deterministic diffs.
///
/// # Returns
///
/// All decodable manifests; malformed files are logged and skipped so one bad
/// app cannot block the compositor from applying the others.
pub fn read_all() -> Vec<ShortcutManifest> {
    read_all_from(&shortcut_dir())
}

/// A shortcut publisher bound to one app id.
#[derive(Debug, Clone)]
pub struct ShortcutClient {
    owner: String,
}

impl ShortcutClient {
    /// Creates a client for `owner`.
    ///
    /// # Arguments
    ///
    /// * `owner` - Stable app id, for example `org.babydrade.topbar`.
    ///
    /// # Returns
    ///
    /// A client whose operations all target that owner's manifest.
    pub fn new(owner: &str) -> Self {
        Self {
            owner: owner.to_string(),
        }
    }

    /// Adds or replaces one binding, keeping bindings with other ids intact.
    ///
    /// # Arguments
    ///
    /// * `binding` - Binding to publish; its `id` is the upsert key.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the new manifest is on disk.
    pub fn register(&self, binding: ShortcutBinding) -> Result<(), ShortcutError> {
        register_in(&shortcut_dir(), &self.owner, binding)
    }

    /// Registers a binding from its parts.
    ///
    /// # Arguments
    ///
    /// * `id` - Stable per-app shortcut id.
    /// * `combo` - Combo in keymap format, for example `W-S-t`.
    /// * `target` - Command or compositor action to run.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the binding is published.
    pub fn bind(&self, id: &str, combo: &str, target: ShortcutTarget) -> Result<(), ShortcutError> {
        self.register(ShortcutBinding {
            id: id.to_string(),
            combo: combo.to_string(),
            target,
            enabled: true,
        })
    }

    /// Removes one binding by id.
    ///
    /// # Arguments
    ///
    /// * `id` - Id of the binding to remove.
    ///
    /// # Returns
    ///
    /// `true` when a binding was removed, `false` when the id was unknown.
    pub fn unregister(&self, id: &str) -> Result<bool, ShortcutError> {
        unregister_in(&shortcut_dir(), &self.owner, id)
    }

    /// Removes every binding owned by this client.
    ///
    /// # Returns
    ///
    /// `Ok(())` even when no manifest was published.
    pub fn clear(&self) -> Result<(), ShortcutError> {
        clear_in(&shortcut_dir(), &self.owner)
    }

    /// Returns this client's currently published bindings.
    ///
    /// # Returns
    ///
    /// An empty vector when nothing has been published.
    pub fn bindings(&self) -> Vec<ShortcutBinding> {
        read_manifest_from(&shortcut_dir(), &self.owner)
            .map(|manifest| manifest.bindings)
            .unwrap_or_default()
    }
}

/// Creates an empty manifest at revision zero for `owner`.
fn new_manifest(owner: &str) -> ShortcutManifest {
    ShortcutManifest {
        owner: owner.to_string(),
        revision: 0,
        bindings: Vec::new(),
    }
}

fn register_in(dir: &Path, owner: &str, binding: ShortcutBinding) -> Result<(), ShortcutError> {
    let mut manifest = read_manifest_from(dir, owner).unwrap_or_else(|| new_manifest(owner));
    manifest
        .bindings
        .retain(|existing| existing.id != binding.id);
    manifest.bindings.push(binding);
    manifest.revision += 1;
    write_to(dir, &manifest)
}

fn unregister_in(dir: &Path, owner: &str, id: &str) -> Result<bool, ShortcutError> {
    let Some(mut manifest) = read_manifest_from(dir, owner) else {
        return Ok(false);
    };
    let before = manifest.bindings.len();
    manifest.bindings.retain(|binding| binding.id != id);
    if manifest.bindings.len() == before {
        return Ok(false);
    }
    manifest.revision += 1;
    write_to(dir, &manifest)?;
    Ok(true)
}

fn clear_in(dir: &Path, owner: &str) -> Result<(), ShortcutError> {
    let path = manifest_path_in(dir, owner)?;
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(ShortcutError::Io(e.to_string())),
    }
}

fn read_manifest_from(dir: &Path, owner: &str) -> Option<ShortcutManifest> {
    let path = manifest_path_in(dir, owner).ok()?;
    let raw = fs::read_to_string(path).ok()?;
    match serde_json::from_str(&raw) {
        Ok(manifest) => Some(manifest),
        Err(e) => {
            tracing::warn!("ignoring malformed shortcut manifest: {e}");
            None
        }
    }
}

fn read_all_from(dir: &Path) -> Vec<ShortcutManifest> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut manifests = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some(MANIFEST_EXT) {
            continue;
        }
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        match serde_json::from_str::<ShortcutManifest>(&raw) {
            Ok(manifest) => manifests.push(manifest),
            Err(e) => tracing::warn!("ignoring malformed shortcut manifest {:?}: {e}", path),
        }
    }
    manifests.sort_by(|a, b| a.owner.cmp(&b.owner));
    manifests
}

fn write_to(dir: &Path, manifest: &ShortcutManifest) -> Result<(), ShortcutError> {
    let path = manifest_path_in(dir, &manifest.owner)?;
    fs::create_dir_all(dir).map_err(|e| ShortcutError::Io(e.to_string()))?;

    let raw = serde_json::to_string(manifest)
        .map_err(|e| ShortcutError::InvalidManifest(e.to_string()))?;
    let temp = path.with_extension(format!("{}.{}.tmp", MANIFEST_EXT, std::process::id()));
    fs::write(&temp, raw).map_err(|e| ShortcutError::Io(e.to_string()))?;
    fs::rename(&temp, &path).map_err(|e| ShortcutError::Io(e.to_string()))
}

/// Maps an owner id to a path-safe, collision-resistant file name.
fn manifest_path_in(dir: &Path, owner: &str) -> Result<PathBuf, ShortcutError> {
    if owner.trim().is_empty() {
        return Err(ShortcutError::InvalidOwner);
    }
    let safe: String = owner
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let name = format!("{safe}-{:016x}.{MANIFEST_EXT}", fnv1a(owner));
    Ok(dir.join(name))
}

fn fnv1a(value: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in value.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(id: &str, combo: &str) -> ShortcutBinding {
        ShortcutBinding {
            id: id.to_string(),
            combo: combo.to_string(),
            target: ShortcutTarget::Command("kitty".into()),
            enabled: true,
        }
    }

    #[test]
    fn register_publishes_a_manifest_with_one_binding() {
        let dir = tempfile::tempdir().unwrap();

        register_in(dir.path(), "app", binding("terminal", "W-S-t")).unwrap();

        let manifest = read_manifest_from(dir.path(), "app").unwrap();
        assert_eq!(manifest.owner, "app");
        assert_eq!(manifest.revision, 1);
        assert_eq!(manifest.bindings, vec![binding("terminal", "W-S-t")]);
    }

    #[test]
    fn registering_same_id_replaces_binding_without_duplicates() {
        let dir = tempfile::tempdir().unwrap();
        register_in(dir.path(), "app", binding("terminal", "W-S-t")).unwrap();

        register_in(dir.path(), "app", binding("terminal", "W-S-e")).unwrap();

        let manifest = read_manifest_from(dir.path(), "app").unwrap();
        assert_eq!(manifest.bindings.len(), 1);
        assert_eq!(manifest.bindings[0].combo, "W-S-e");
        assert_eq!(manifest.revision, 2);
    }

    #[test]
    fn unregister_removes_one_binding_and_clear_removes_the_manifest() {
        let dir = tempfile::tempdir().unwrap();
        register_in(dir.path(), "app", binding("terminal", "W-S-t")).unwrap();
        register_in(dir.path(), "app", binding("power", "A-F4")).unwrap();

        assert!(unregister_in(dir.path(), "app", "terminal").unwrap());
        assert!(!unregister_in(dir.path(), "app", "missing").unwrap());
        assert_eq!(
            read_manifest_from(dir.path(), "app")
                .unwrap()
                .bindings
                .len(),
            1
        );

        clear_in(dir.path(), "app").unwrap();
        assert!(read_manifest_from(dir.path(), "app").is_none());
        clear_in(dir.path(), "app").unwrap();
    }

    #[test]
    fn read_all_orders_manifests_by_owner_and_skips_bad_files() {
        let dir = tempfile::tempdir().unwrap();
        for owner in ["zeta", "alpha"] {
            write_to(dir.path(), &new_manifest(owner)).unwrap();
        }
        fs::write(dir.path().join("broken.json"), "not json").unwrap();

        let owners: Vec<_> = read_all_from(dir.path())
            .into_iter()
            .map(|manifest| manifest.owner)
            .collect();
        assert_eq!(owners, vec!["alpha", "zeta"]);
    }

    #[test]
    fn owner_ids_cannot_escape_the_manifest_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = manifest_path_in(dir.path(), "../../etc/passwd").unwrap();
        assert_eq!(path.parent(), Some(dir.path()));
        assert!(manifest_path_in(dir.path(), "  ").is_err());
    }
}
