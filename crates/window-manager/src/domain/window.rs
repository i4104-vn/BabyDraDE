//! Window facts shared across layers (pure data + in-memory cache).
//!
//! The adapter extracts protocol data into these types; the domain never
//! touches a `Window` handle, config file or system service.

use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

use smithay::utils::{Logical, Rectangle};

/// Per-frame visual values the renderer applies to one window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WindowVisualState {
    pub(crate) active: bool,
    pub(crate) scale_factor: f64,
    pub(crate) alpha_factor: f32,
    pub(crate) target_rect: Option<Rectangle<f64, Logical>>,
}

impl WindowVisualState {
    /// Returns the neutral visual state with no animation applied.
    pub(crate) fn idle() -> Self {
        Self {
            active: false,
            scale_factor: 1.0,
            alpha_factor: 1.0,
            target_rect: None,
        }
    }
}

/// Identity and display name extracted from one Wayland toplevel.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct WindowMetadata {
    pub(crate) app_id: String,
    pub(crate) title: String,
    pub(crate) app_name: String,
}

#[derive(Default)]
struct MetadataCache(HashMap<String, WindowMetadata>);

static CACHE: OnceLock<RwLock<MetadataCache>> = OnceLock::new();

fn cache() -> &'static RwLock<MetadataCache> {
    CACHE.get_or_init(|| RwLock::new(MetadataCache::default()))
}

/// Stores metadata under an adapter-provided surface key.
///
/// # Arguments
///
/// * `key` - Opaque key identifying the window surface.
/// * `metadata` - Metadata to cache.
pub(crate) fn store(key: &str, metadata: WindowMetadata) {
    if let Ok(mut cache) = cache().write() {
        cache.0.insert(key.to_string(), metadata);
    }
}

/// Returns cached metadata for a surface.
///
/// # Arguments
///
/// * `key` - Opaque key identifying the window surface.
///
/// # Returns
///
/// The cached metadata, or `None` when the surface is unknown.
pub(crate) fn cached(key: &str) -> Option<WindowMetadata> {
    cache().read().ok()?.0.get(key).cloned()
}

/// Removes cached metadata for a destroyed surface.
///
/// # Arguments
///
/// * `key` - Opaque key identifying the window surface.
pub(crate) fn forget(key: &str) {
    if let Ok(mut cache) = cache().write() {
        cache.0.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caches_and_forgets_metadata() {
        store(
            "surface-1",
            WindowMetadata {
                app_id: "app".into(),
                title: "Title".into(),
                app_name: "App".into(),
            },
        );
        assert_eq!(cached("surface-1").map(|m| m.app_id), Some("app".into()));
        forget("surface-1");
        assert!(cached("surface-1").is_none());
    }
}
