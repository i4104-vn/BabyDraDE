//! Launcher overlay state model and construction helpers.

use gtk4::gio::{prelude::*, AppInfo};
use smithay::backend::renderer::element::memory::MemoryRenderBuffer;

use crate::modules::shell::launcher::search::{
    SearchBackdropKey, SearchPanelKey, SearchRegion, SearchState,
};
use crate::modules::shell::launcher::{BackdropKey, BufferRegion, CacheKey, LauncherApp};
use ui_kit::components::dock::{DockLayout, DockSide, DockState};

pub(crate) struct LauncherState {
    pub(crate) apps: Vec<LauncherApp>,
    pub(crate) visible: bool,
    pub(crate) dock: DockState,
    /// Vertical centre of the capsule in logical coordinates.
    pub(crate) center_y: f64,
    pub(crate) buffer: Option<MemoryRenderBuffer>,
    pub(crate) region: Option<BufferRegion>,
    pub(crate) cache_key: Option<CacheKey>,
    pub(crate) backdrop_buffer: Option<MemoryRenderBuffer>,
    pub(crate) backdrop_size: Option<(i32, i32)>,
    pub(crate) backdrop_cache_key: Option<BackdropKey>,
    /// Expanded search overlay (query, results, fly-in/fade animations).
    pub(crate) search: SearchState,
    /// Off-screen surfaces: the centered panel and its full-screen dimmer.
    pub(crate) search_panel_buffer: Option<MemoryRenderBuffer>,
    pub(crate) search_panel_region: Option<SearchRegion>,
    pub(crate) search_panel_key: Option<SearchPanelKey>,
    pub(crate) search_backdrop_buffer: Option<MemoryRenderBuffer>,
    pub(crate) search_backdrop_size: Option<(i32, i32)>,
    pub(crate) search_backdrop_key: Option<SearchBackdropKey>,
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

    pub(crate) fn panel(&self) -> DockLayout {
        DockLayout::new(self.apps.len(), self.center_y, 0.0, DockSide::Left)
    }

    pub(crate) fn slide_offset(&self) -> f64 {
        self.dock.slide_offset(DockSide::Left)
    }
}
