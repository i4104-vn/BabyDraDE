//! Extracts window identity from Wayland protocol state and caches it in the
//! domain. Protocol access lives here so `domain::window` stays pure.

use smithay::{
    desktop::Window,
    reexports::wayland_server::Resource,
    wayland::{compositor::with_states, shell::xdg::XdgToplevelSurfaceData},
};

use crate::domain::window::WindowMetadata;

fn key(window: &Window) -> Option<String> {
    Some(format!("{:?}", window.toplevel()?.wl_surface().id()))
}

/// Extracts and caches a window's current identity from protocol state.
///
/// # Arguments
///
/// * `window` - Window whose toplevel provides the app id and title.
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

    let app_name = services::system::window::get_app_display_name(&app_id, &title);
    crate::domain::window::store(
        &key,
        WindowMetadata {
            app_id,
            title,
            app_name,
        },
    );
}

/// Returns cached metadata for a window, extracting it on first use.
///
/// # Arguments
///
/// * `window` - Window to look up.
///
/// # Returns
///
/// The metadata, or `None` when the window has no toplevel surface.
pub(crate) fn get(window: &Window) -> Option<WindowMetadata> {
    let key = key(window)?;
    if let Some(metadata) = crate::domain::window::cached(&key) {
        return Some(metadata);
    }
    refresh(window);
    crate::domain::window::cached(&key)
}

/// Drops cached metadata for a destroyed window.
///
/// # Arguments
///
/// * `window` - Window whose entry should be removed.
pub(crate) fn remove(window: &Window) {
    if let Some(key) = key(window) {
        crate::domain::window::forget(&key);
    }
}
