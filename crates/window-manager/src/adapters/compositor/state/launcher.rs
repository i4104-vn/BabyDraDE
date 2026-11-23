//! Launcher state: app list, input and the render entry point. Dock geometry,
//! selection animation and shared drawing live in `ui_kit::components::dock`.

use animation::NAVIGATION_DURATION;
use gtk4::gio::{prelude::*, AppInfo, File as GioFile};
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::element::{
            memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
            Kind,
        },
    },
    utils::{Logical, Physical, Point, Scale, Transform},
};

use crate::adapters::rendering::buffer::render_pixels;
use crate::adapters::rendering::CustomRenderElements;
use ui_kit::components::dock::DockSide;

use crate::adapters::desktop::launcher::buffer::{
    buffer_damage, physical_region, render_backdrop_pixels,
};
use crate::adapters::desktop::launcher::draw::draw_scene;
use crate::adapters::desktop::launcher::search::buffer::{
    buffer_damage as search_buffer_damage, panel_region,
    render_backdrop_pixels as render_search_backdrop,
};
use crate::adapters::desktop::launcher::search::draw::draw_panel;
use crate::adapters::desktop::launcher::search::index::url_encode;
use crate::adapters::desktop::launcher::search::layout::SearchLayout;
use crate::adapters::desktop::launcher::search::{
    SearchAction, SearchBackdropKey, SearchPanelKey, SearchRegion, SearchState,
};
use crate::adapters::desktop::launcher::{
    BackdropKey, BufferRegion, CacheKey, LauncherApp, BACKDROP_WIDTH_MIN, BACKDROP_WIDTH_RATIO,
};

use ui_kit::components::dock::{DockLayout, DockState};

use common::constants::launcher::SEARCH_BACKDROP_ALPHA;

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

    pub(crate) fn is_visible(&self) -> bool {
        self.visible && !self.dock.is_closing()
    }

    pub(crate) fn toggle(&mut self) -> bool {
        if self.visible && !self.dock.is_closing() {
            self.close();
        } else {
            let selected = self
                .dock
                .selected
                .unwrap_or(0)
                .min(self.apps.len().saturating_sub(1));
            self.visible = true;
            self.dock.open(selected, true);
            self.cache_key = None;
            // A fresh opening always starts from the plain docked capsule.
            self.search.reset();
            self.search_panel_key = None;
            self.search_backdrop_key = None;
        }
        self.visible
    }

    pub(crate) fn close(&mut self) {
        if !self.visible {
            return;
        }
        self.dock.begin_close();
        self.search.clear();
        self.cache_key = None;
    }

    fn finish_close(&mut self) {
        self.visible = false;
        self.dock.finish_close();
        self.search.reset();
        self.cache_key = None;
        self.backdrop_cache_key = None;
        self.search_panel_key = None;
        self.search_backdrop_key = None;
    }

    pub(crate) fn arrow_pressed(&mut self, direction: i32) -> bool {
        self.visible && !self.search.is_active() && self.dock.key_pressed(direction)
    }

    pub(crate) fn arrow_released(&mut self, direction: i32) {
        self.dock.key_released(direction);
    }

    pub(crate) fn move_selection(&mut self, delta: i32) {
        if self.search.is_active() {
            return;
        }
        self.dock
            .move_selection(self.apps.len(), delta, NAVIGATION_DURATION);
        self.cache_key = None;
    }

    pub(crate) fn activate(&mut self) {
        if self.search.is_active() {
            self.search_activate();
            return;
        }
        if !self.dock.is_closing() {
            if let Some(index) = self.dock.display_selected() {
                self.launch(index);
            }
        }
    }

    pub(crate) fn scroll_by(&mut self, slots: f64) {
        if self.search.is_active() {
            return;
        }
        self.dock.scroll_by(self.apps.len(), slots);
        self.cache_key = None;
    }

    pub(crate) fn drag_by(&mut self, slots: f64) {
        if self.search.is_active() {
            return;
        }
        self.dock.drag_by(self.apps.len(), slots);
        self.cache_key = None;
    }

    pub(crate) fn finish_drag(&mut self) {
        if self.search.is_active() {
            return;
        }
        self.dock.finish_drag(self.apps.len());
        self.cache_key = None;
    }

    pub(crate) fn index_at(&self, point: Point<f64, Logical>) -> Option<usize> {
        if !self.visible || self.dock.is_closing() || self.search.is_active() {
            return None;
        }
        self.panel().index_at(
            (point.x, point.y),
            self.slide_offset(),
            self.apps.len(),
            self.dock.position(),
        )
    }

    pub(crate) fn is_over_panel(&self, point: Point<f64, Logical>) -> bool {
        self.visible
            && !self.dock.is_closing()
            && !self.search.is_active()
            && self
                .panel()
                .contains((point.x, point.y), self.slide_offset())
    }

    pub(crate) fn update_hover(&mut self, point: Point<f64, Logical>) -> bool {
        if self.search.is_active() {
            return false;
        }
        let Some(index) = self.index_at(point) else {
            if self.dock.hovered.is_some() {
                self.dock.clear_hover();
                return true;
            }
            return false;
        };
        if self.dock.hovered == Some(index) {
            return false;
        }
        // Hover changes only the highlight; it must not retarget `position`,
        // otherwise the carousel moves and hit-testing starts to chase itself.
        self.dock.set_hover(index);
        true
    }

    // -- search overlay ----------------------------------------------------

    pub(crate) fn search_is_active(&self) -> bool {
        self.visible && self.search.is_active()
    }

    /// A printable key becomes the first character of the query, which reveals
    /// the centered search surface and fills all three result categories.
    pub(crate) fn search_input(&mut self, character: char) {
        if !self.visible || self.dock.is_closing() {
            return;
        }
        self.search.push_char(character, &self.apps);
    }

    pub(crate) fn search_backspace(&mut self) {
        if !self.visible {
            return;
        }
        self.search.backspace(&self.apps);
    }

    pub(crate) fn search_move(&mut self, delta: i32) {
        if self.search_is_active() {
            self.search.move_selection(delta);
        }
    }

    /// Escape returns to the docked launcher; a second Escape closes it.
    pub(crate) fn search_escape(&mut self) -> bool {
        if !self.search_is_active() {
            return false;
        }
        self.search.clear();
        true
    }

    pub(crate) fn search_activate(&mut self) {
        let Some(action) = self.search.activate() else {
            return;
        };
        match action {
            SearchAction::App(index) => self.launch(index),
            SearchAction::Path(path) => {
                let uri = GioFile::for_path(&path).uri().to_string();
                let _ = AppInfo::launch_default_for_uri(&uri, None::<&gtk4::gio::AppLaunchContext>);
                self.close();
            }
            SearchAction::Web(query) => {
                let url = format!("https://www.google.com/search?q={}", url_encode(&query));
                let _ = AppInfo::launch_default_for_uri(&url, None::<&gtk4::gio::AppLaunchContext>);
                self.close();
            }
        }
    }

    /// Handles a click while the search surface owns the screen. Returns true
    /// when the click was consumed (result launched, or dismissed back to dock).
    pub(crate) fn search_click(&mut self, point: Point<f64, Logical>) -> bool {
        if !self.search_is_active() {
            return false;
        }
        if let Some(index) = self.search.item_at(point) {
            self.search.select(index);
            self.search_activate();
        } else if !self.search.panel_contains(point) {
            self.search.clear();
        }
        true
    }

    pub(crate) fn launch(&mut self, index: usize) {
        if let Some(app) = self.apps.get(index) {
            let _ = app.info.launch(&[], None::<&gtk4::gio::AppLaunchContext>);
        }
        self.close();
    }

    pub(crate) fn render(
        &mut self,
        renderer: &mut smithay::backend::renderer::glow::GlowRenderer,
        output_size: (i32, i32),
        scale: Scale<f64>,
    ) -> Vec<CustomRenderElements> {
        if !self.visible || self.apps.is_empty() {
            return Vec::new();
        }

        self.dock.tick_key_repeat(self.apps.len());
        if self.dock.is_closing() && !self.dock.presence.is_animating() {
            self.finish_close();
            return Vec::new();
        }

        let (screen_w, screen_h) = (output_size.0.max(1), output_size.1.max(1));
        let logical_w = (screen_w as f64 / scale.x).max(1.0);
        let logical_h = (screen_h as f64 / scale.y).max(1.0);
        self.center_y = logical_h / 2.0;

        // The search overlay fades the docked capsule out in place as it flies
        // in, and fades it back in on the way out. Both sides share the same
        // clock-driven tweens, so the crossfade cannot drop frames.
        let search_presence = self.search.presence();
        let search_tick = self.search.animation_tick();
        let dock_tick = self.dock.animation_tick();
        let dock_presence = (self.dock.presence() * (1.0 - search_presence)).clamp(0.0, 1.0);

        let panel = self.panel();
        let region = physical_region(&panel, scale, screen_w, screen_h);
        let slide_x = self.slide_offset();
        let position = self.dock.position();
        let highlight_position = self.dock.display_position();
        let selected = self.dock.display_selected();
        let key = (
            self.apps.len(),
            selected,
            dock_tick,
            search_tick,
            region,
            dock_presence.to_bits(),
        );

        if self.buffer.is_none() || self.region != Some(region) {
            self.buffer = Some(MemoryRenderBuffer::new(
                Fourcc::Argb8888,
                (region.2, region.3),
                1,
                Transform::Normal,
                None,
            ));
            self.region = Some(region);
            self.cache_key = None;
        }

        if dock_presence > 0.001 && self.cache_key != Some(key) {
            let apps = &self.apps;
            if let Some(pixels) =
                render_pixels(region.2, region.3, scale, (region.0, region.1), |ctx| {
                    draw_scene(
                        ctx,
                        apps,
                        &panel,
                        position,
                        highlight_position,
                        selected,
                        logical_w,
                        dock_presence,
                        slide_x,
                    )
                })
            {
                if let Some(buffer) = self.buffer.as_mut() {
                    let mut context = buffer.render();
                    let _ = context.draw(|dst| {
                        if dst.len() != pixels.len() {
                            return Err(());
                        }
                        dst.copy_from_slice(&pixels);
                        Ok(buffer_damage(region.2, region.3))
                    });
                    self.cache_key = Some(key);
                }
            }
        }

        let gradient_w = (logical_w * BACKDROP_WIDTH_RATIO)
            .max(BACKDROP_WIDTH_MIN)
            .min(logical_w);
        let backdrop_w = ((gradient_w * scale.x).ceil() as i32).clamp(1, screen_w);
        let backdrop_size = (backdrop_w, screen_h);
        let presence_tick = if self.dock.presence.is_animating() {
            self.dock.presence.frame_tick()
        } else {
            u64::MAX
        };
        let backdrop_key = (
            backdrop_size,
            presence_tick,
            search_tick,
            dock_presence.to_bits(),
        );
        if self.backdrop_buffer.is_none()
            || self.backdrop_size != Some(backdrop_size)
            || self.backdrop_cache_key != Some(backdrop_key)
        {
            if let Some(pixels) =
                render_backdrop_pixels(backdrop_w, screen_h, scale, gradient_w, dock_presence)
            {
                let mut buffer = MemoryRenderBuffer::new(
                    Fourcc::Argb8888,
                    backdrop_size,
                    1,
                    Transform::Normal,
                    None,
                );
                {
                    let mut context = buffer.render();
                    let _ = context.draw(|dst| {
                        if dst.len() != pixels.len() {
                            return Err(());
                        }
                        dst.copy_from_slice(&pixels);
                        Ok(buffer_damage(backdrop_w, screen_h))
                    });
                }
                self.backdrop_buffer = Some(buffer);
                self.backdrop_size = Some(backdrop_size);
                self.backdrop_cache_key = Some(backdrop_key);
            }
        }

        // -- search surface -------------------------------------------------
        let layout = SearchLayout::compute(
            logical_w,
            logical_h,
            self.search.content_height(),
            self.search.fly_offset(),
        );
        self.search.set_layout(layout);
        let search_region = panel_region(&layout, scale, screen_w, screen_h);
        let panel_key = self.search.panel_key(search_region);

        if self.search_panel_buffer.is_none() || self.search_panel_region != Some(search_region) {
            self.search_panel_buffer = Some(MemoryRenderBuffer::new(
                Fourcc::Argb8888,
                (search_region.2, search_region.3),
                1,
                Transform::Normal,
                None,
            ));
            self.search_panel_region = Some(search_region);
            self.search_panel_key = None;
        }

        if search_presence > 0.001 && self.search_panel_key != Some(panel_key) {
            let search = &self.search;
            if let Some(pixels) = render_pixels(
                search_region.2,
                search_region.3,
                scale,
                (search_region.0, search_region.1),
                |ctx| draw_panel(ctx, search, &layout, search_presence),
            ) {
                if let Some(buffer) = self.search_panel_buffer.as_mut() {
                    let mut context = buffer.render();
                    let _ = context.draw(|dst| {
                        if dst.len() != pixels.len() {
                            return Err(());
                        }
                        dst.copy_from_slice(&pixels);
                        Ok(search_buffer_damage(search_region.2, search_region.3))
                    });
                    self.search_panel_key = Some(panel_key);
                }
            }
        }

        let search_backdrop_size = (screen_w, screen_h);
        let search_backdrop_key = (search_backdrop_size, search_tick, search_presence.to_bits());
        if search_presence > 0.001
            && (self.search_backdrop_buffer.is_none()
                || self.search_backdrop_size != Some(search_backdrop_size)
                || self.search_backdrop_key != Some(search_backdrop_key))
        {
            if let Some(pixels) = render_search_backdrop(
                screen_w,
                screen_h,
                scale,
                SEARCH_BACKDROP_ALPHA * search_presence,
            ) {
                let mut buffer = MemoryRenderBuffer::new(
                    Fourcc::Argb8888,
                    search_backdrop_size,
                    1,
                    Transform::Normal,
                    None,
                );
                {
                    let mut context = buffer.render();
                    let _ = context.draw(|dst| {
                        if dst.len() != pixels.len() {
                            return Err(());
                        }
                        dst.copy_from_slice(&pixels);
                        Ok(search_buffer_damage(screen_w, screen_h))
                    });
                }
                self.search_backdrop_buffer = Some(buffer);
                self.search_backdrop_size = Some(search_backdrop_size);
                self.search_backdrop_key = Some(search_backdrop_key);
            }
        }

        // Front-to-back: search panel, its dimmer, then the fading dock.
        let mut elements = Vec::with_capacity(4);
        if search_presence > 0.001 {
            if let Some(buffer) = self.search_panel_buffer.as_ref() {
                if let Ok(elem) = MemoryRenderBufferRenderElement::from_buffer(
                    renderer,
                    Point::<f64, Physical>::from((search_region.0 as f64, search_region.1 as f64)),
                    buffer,
                    None,
                    None,
                    None,
                    Kind::Unspecified,
                ) {
                    elements.push(CustomRenderElements::Memory(elem));
                }
            }
            if let Some(backdrop) = self.search_backdrop_buffer.as_ref() {
                if let Ok(elem) = MemoryRenderBufferRenderElement::from_buffer(
                    renderer,
                    Point::<f64, Physical>::from((0.0, 0.0)),
                    backdrop,
                    None,
                    None,
                    None,
                    Kind::Unspecified,
                ) {
                    elements.push(CustomRenderElements::Memory(elem));
                }
            }
        }
        if dock_presence > 0.001 {
            if let Some(buffer) = self.buffer.as_ref() {
                if let Ok(elem) = MemoryRenderBufferRenderElement::from_buffer(
                    renderer,
                    Point::<f64, Physical>::from((region.0 as f64, region.1 as f64)),
                    buffer,
                    None,
                    None,
                    None,
                    Kind::Unspecified,
                ) {
                    elements.push(CustomRenderElements::Memory(elem));
                }
            }
            if let Some(backdrop) = self.backdrop_buffer.as_ref() {
                if let Ok(elem) = MemoryRenderBufferRenderElement::from_buffer(
                    renderer,
                    Point::<f64, Physical>::from((0.0, 0.0)),
                    backdrop,
                    None,
                    None,
                    None,
                    Kind::Unspecified,
                ) {
                    elements.push(CustomRenderElements::Memory(elem));
                }
            }
        }
        elements
    }
}
