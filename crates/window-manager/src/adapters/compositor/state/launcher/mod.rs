//! Launcher state: app discovery, dock geometry and the render entry point.
//! Input handling lives in [`input`]; buffer assembly lives in [`render`].

use gtk4::gio::{prelude::*, AppInfo};
use smithay::backend::renderer::element::memory::MemoryRenderBuffer;

use crate::adapters::desktop::launcher::search::{
    SearchBackdropKey, SearchPanelKey, SearchRegion, SearchState,
};
use crate::adapters::desktop::launcher::{BackdropKey, BufferRegion, CacheKey, LauncherApp};
use ui_kit::components::dock::{DockLayout, DockSide, DockState};

mod input;
mod render;

pub(crate) struct LauncherState {
    apps: Vec<LauncherApp>,
    visible: bool,
    dock: DockState,
    /// Vertical centre of the capsule in logical coordinates.
    center_y: f64,
    buffer: Option<MemoryRenderBuffer>,
    region: Option<BufferRegion>,
    cache_key: Option<CacheKey>,
    backdrop_buffer: Option<MemoryRenderBuffer>,
    backdrop_size: Option<(i32, i32)>,
    backdrop_cache_key: Option<BackdropKey>,
    /// Expanded search overlay (query, results, fly-in/fade animations).
    search: SearchState,
    /// Off-screen surfaces: the centered panel and its full-screen dimmer.
    search_panel_buffer: Option<MemoryRenderBuffer>,
    search_panel_region: Option<SearchRegion>,
    search_panel_key: Option<SearchPanelKey>,
    search_backdrop_buffer: Option<MemoryRenderBuffer>,
    search_backdrop_size: Option<(i32, i32)>,
    search_backdrop_key: Option<SearchBackdropKey>,
}

impl Default for LauncherState {
    fn default() -> Self {
        Self::new()
    }
}

impl LauncherState {
    pub(crate) fn new() -> Self {
        let mut apps: Vec<LauncherApp> = AppInfo::all()
            .into_iter()
            .filter(|info| info.should_show())
            .map(|info| {
                let id = info.id().map(|id| id.to_string()).unwrap_or_default();
                LauncherApp {
                    name: info.display_name().to_string(),
                    icon: ui_kit::get_app_icon_pixbuf_any(&id, 96),
                    info,
                }
            })
            .collect();
        apps.sort_by_key(|app| app.name.to_lowercase());

        Self {
            apps,
            visible: false,
            dock: DockState::new(),
            center_y: 0.0,
            buffer: None,
            region: None,
            cache_key: None,
            backdrop_buffer: None,
            backdrop_size: None,
            backdrop_cache_key: None,
            search: SearchState::new(),
            search_panel_buffer: None,
            search_panel_region: None,
            search_panel_key: None,
            search_backdrop_buffer: None,
            search_backdrop_size: None,
            search_backdrop_key: None,
        }
    }

    fn panel(&self) -> DockLayout {
        DockLayout::new(self.apps.len(), self.center_y, 0.0, DockSide::Left)
    }

    fn slide_offset(&self) -> f64 {
        self.dock.slide_offset(DockSide::Left)
    }
}
