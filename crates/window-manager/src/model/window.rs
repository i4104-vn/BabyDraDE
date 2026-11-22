//! Plain window identity metadata shared across layers.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use smithay::{
    desktop::Window,
    reexports::wayland_server::{backend::ObjectId, Resource},
    wayland::{compositor::with_states, shell::xdg::XdgToplevelSurfaceData},
};

/// Identity and display data extracted once from a Wayland toplevel.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WindowMetadata {
    pub app_id: String,
    pub title: String,
    pub app_name: String,
}

type MetadataCache = RwLock<HashMap<ObjectId, WindowMetadata>>;

static CACHE: OnceLock<MetadataCache> = OnceLock::new();

fn cache() -> &'static MetadataCache {
    CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn key(window: &Window) -> Option<ObjectId> {
    Some(window.toplevel()?.wl_surface().id())
}

/// Refreshes the cached metadata from the current toplevel state.
pub fn refresh(window: &Window) {
    let Some(id) = key(window) else {
        return;
    };
    let Some(toplevel) = window.toplevel() else {
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

    let app_name = services::system::window::get_app_display_name(&app_id, &title);
    if let Ok(mut map) = cache().write() {
        map.insert(
            id,
            WindowMetadata {
                app_id,
                title,
                app_name,
            },
        );
    }
}

/// Returns cached metadata, refreshing once when the window is unknown.
pub fn get(window: &Window) -> Option<WindowMetadata> {
    let id = key(window)?;
    if let Some(metadata) = cache().read().ok()?.get(&id) {
        return Some(metadata.clone());
    }
    refresh(window);
    cache().read().ok()?.get(&id).cloned()
}

/// Drops cached metadata when a window is destroyed.
pub fn remove(window: &Window) {
    if let Some(id) = key(window) {
        if let Ok(mut map) = cache().write() {
            map.remove(&id);
        }
    }
}
