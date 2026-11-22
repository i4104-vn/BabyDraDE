//! Launcher state: app list, input and the render entry point. Dock geometry,
//! selection animation and shared drawing live in `ui_kit::components::dock`.

use animation::NAVIGATION_DURATION;
use gtk4::gio::{prelude::*, AppInfo};
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

use crate::rendering::buffer::render_pixels;
use crate::rendering::CustomRenderElements;
use ui_kit::components::dock::DockSide;

use super::buffer::{buffer_damage, physical_region, render_backdrop_pixels};
use super::draw::draw_scene;
use super::{
    BackdropKey, BufferRegion, CacheKey, LauncherApp, BACKDROP_WIDTH_MIN, BACKDROP_WIDTH_RATIO,
};

use ui_kit::components::dock::{DockLayout, DockState};

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
        }
        self.visible
    }

    pub(crate) fn close(&mut self) {
        if !self.visible {
            return;
        }
        self.dock.begin_close();
        self.cache_key = None;
    }

    fn finish_close(&mut self) {
        self.visible = false;
        self.dock.finish_close();
        self.cache_key = None;
        self.backdrop_cache_key = None;
    }

    pub(crate) fn arrow_pressed(&mut self, direction: i32) -> bool {
        self.visible && self.dock.key_pressed(direction)
    }

    pub(crate) fn arrow_released(&mut self, direction: i32) {
        self.dock.key_released(direction);
    }

    pub(crate) fn move_selection(&mut self, delta: i32) {
        self.dock
            .move_selection(self.apps.len(), delta, NAVIGATION_DURATION);
        self.cache_key = None;
    }

    pub(crate) fn activate(&mut self) {
        if !self.dock.is_closing() {
            if let Some(index) = self.dock.display_selected() {
                self.launch(index);
            }
        }
    }

    pub(crate) fn scroll_by(&mut self, slots: f64) {
        self.dock.scroll_by(self.apps.len(), slots);
        self.cache_key = None;
    }

    pub(crate) fn drag_by(&mut self, slots: f64) {
        self.dock.drag_by(self.apps.len(), slots);
        self.cache_key = None;
    }

    pub(crate) fn finish_drag(&mut self) {
        self.dock.finish_drag(self.apps.len());
        self.cache_key = None;
    }

    pub(crate) fn index_at(&self, point: Point<f64, Logical>) -> Option<usize> {
        if !self.visible || self.dock.is_closing() {
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
            && self
                .panel()
                .contains((point.x, point.y), self.slide_offset())
    }

    pub(crate) fn update_hover(&mut self, point: Point<f64, Logical>) -> bool {
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

        let panel = self.panel();
        let region = physical_region(&panel, scale, screen_w, screen_h);
        let tick = self.dock.animation_tick();
        let presence = self.dock.presence();
        let slide_x = self.slide_offset();
        let position = self.dock.position();
        let highlight_position = self.dock.display_position();
        let selected = self.dock.display_selected();
        let key = (self.apps.len(), selected, tick, region);

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

        if self.cache_key != Some(key) {
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
                        presence,
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
        let backdrop_key = (backdrop_size, presence_tick);
        if self.backdrop_buffer.is_none()
            || self.backdrop_size != Some(backdrop_size)
            || self.backdrop_cache_key != Some(backdrop_key)
        {
            if let Some(pixels) =
                render_backdrop_pixels(backdrop_w, screen_h, scale, gradient_w, presence)
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

        let mut elements = Vec::with_capacity(2);
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
        elements
    }
}
