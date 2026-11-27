//! Extracts and caches window identity from Wayland protocol state.
//!
//! This cache lives in the adapter layer, not in `domain`: it is process state
//! with a lifecycle, not a pure rule. Keeping it here lets both the compositor
//! and the renderer query metadata without importing each other.

use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

use smithay::{
    desktop::Window,
    reexports::wayland_server::Resource,
    wayland::{compositor::with_states, shell::xdg::XdgToplevelSurfaceData},
};

use crate::shared::types::window::WindowMetadata;

#[derive(Default)]
struct MetadataCache(HashMap<String, WindowMetadata>);

static CACHE: OnceLock<RwLock<MetadataCache>> = OnceLock::new();

fn cache() -> &'static RwLock<MetadataCache> {
    CACHE.get_or_init(|| RwLock::new(MetadataCache::default()))
}

fn key(window: &Window) -> Option<String> {
    Some(format!("{:?}", window.toplevel()?.wl_surface().id()))
}

/// Extracts and caches a window's current identity from protocol state.
pub(crate) fn refresh(window: &Window) {
    let (Some(key), Some(toplevel)) = (key(window), window.toplevel()) else {
        return;
    };
    let (app_id, title) = with_states(toplevel.wl_surface(), |states| {
        states
            .data_map
            .get::<XdgToplevelSurfaceData>()
            .and_then(|data| data.lock().ok())
            .map(|guard| {
                (
                    guard.app_id.clone().unwrap_or_default(),
                    guard.title.clone().unwrap_or_default(),
                )
            })
    })
    .unwrap_or_default();

    // A shared app id is disambiguated from the title so a window is not
    // labelled as the shell it happens to run on.
    let app_name = services::system::window::get_app_display_name(&app_id, &title);
    if let Ok(mut cache) = cache().write() {
        cache.0.insert(
            key,
            WindowMetadata {
                app_id,
                title,
                app_name,
            },
        );
    }
}

/// Returns cached metadata for a window, extracting it on first use.
pub(crate) fn get(window: &Window) -> Option<WindowMetadata> {
    let key = key(window)?;
    if let Some(metadata) = cache().read().ok()?.0.get(&key).cloned() {
        return Some(metadata);
    }
    refresh(window);
    cache().read().ok()?.0.get(&key).cloned()
}

/// Drops cached metadata for a destroyed window.
pub(crate) fn remove(window: &Window) {
    if let Some(key) = key(window) {
        if let Ok(mut cache) = cache().write() {
            cache.0.remove(&key);
        }
    }
}
