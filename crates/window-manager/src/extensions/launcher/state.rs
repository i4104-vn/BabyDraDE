//! Launcher state: app list, selection, pointer/wheel/keyboard input and the
//! render entry point. Drawing and geometry live in sibling modules.

use animation::{Easing, Tween};
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
use std::time::Duration;

use crate::rendering::CustomRenderElements;

use super::buffer::{buffer_damage, physical_region, render_backdrop_pixels, render_pixels};
use super::draw::draw_scene;
use super::layout::{visual_center, Panel};
use super::{
    BufferRegion, CacheKey, LauncherApp, BACKDROP_WIDTH_MAX, BACKDROP_WIDTH_MIN,
    BACKDROP_WIDTH_RATIO, MOVE_ANIM_MS, SCROLL_ANIM_MS, SLOT, VISIBLE_SLOTS,
};

/// State of the launcher.
pub(crate) struct LauncherState {
    apps: Vec<LauncherApp>,
    visible: bool,
    /// Continuous selection position (index, fractional while animating).
    position: Tween,
    selected: Option<usize>,
    /// Vertical centre of the capsule in logical coordinates.
    center_y: f64,
    buffer: Option<MemoryRenderBuffer>,
    region: Option<BufferRegion>,
    cache_key: Option<CacheKey>,
    backdrop_buffer: Option<MemoryRenderBuffer>,
    backdrop_size: Option<(i32, i32)>,
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
            position: Tween::settled(0.0),
            selected: None,
            center_y: 0.0,
            buffer: None,
            region: None,
            cache_key: None,
            backdrop_buffer: None,
            backdrop_size: None,
        }
    }

    fn panel(&self) -> Panel {
        Panel::new(self.apps.len(), self.center_y)
    }

    pub(crate) fn is_visible(&self) -> bool {
        self.visible
    }

    /// Opens the launcher with the current app selected.
    pub(crate) fn toggle(&mut self) -> bool {
        self.visible = !self.visible;
        if self.visible {
            let selected = self
                .selected
                .unwrap_or(0)
                .min(self.apps.len().saturating_sub(1));
            self.selected = Some(selected);
            self.position = Tween::settled(selected as f64);
        } else {
            self.close();
        }
        self.cache_key = None;
        self.visible
    }

    pub(crate) fn close(&mut self) {
        self.visible = false;
        self.selected = None;
        self.position = Tween::settled(0.0);
        self.cache_key = None;
    }

    /// Moves the keyboard selection with a smooth transition.
    pub(crate) fn move_selection(&mut self, delta: i32) {
        if self.apps.is_empty() {
            return;
        }
        let current = self.selected.unwrap_or(0) as i32;
        let next = (current + delta).clamp(0, self.apps.len() as i32 - 1) as usize;
        self.position.retarget(
            next as f64,
            Duration::from_millis(MOVE_ANIM_MS),
            Easing::EaseOutCubic,
        );
        self.selected = Some(next);
        self.cache_key = None;
    }

    pub(crate) fn activate(&mut self) {
        if let Some(index) = self.selected {
            self.launch(index);
        }
    }

    pub(crate) fn scroll_by(&mut self, slots: f64) {
        if self.apps.is_empty() {
            return;
        }
        let current = self.position.target();
        let next = (current + slots)
            .round()
            .clamp(0.0, self.apps.len() as f64 - 1.0);
        if (next - current).abs() > f64::EPSILON {
            self.selected = Some(next as usize);
            self.position.retarget(
                next,
                Duration::from_millis(SCROLL_ANIM_MS),
                Easing::EaseOutCubic,
            );
            self.cache_key = None;
        }
    }

    pub(crate) fn index_at(&self, point: Point<f64, Logical>) -> Option<usize> {
        if !self.visible || self.apps.is_empty() {
            return None;
        }
        let panel = self.panel();
        if !panel.contains(point) {
            return None;
        }

        let visual = visual_center(self.apps.len(), self.position.value());
        let index = (visual + (point.y - self.center_y) / SLOT).round();
        if index < 0.0 || index >= self.apps.len() as f64 {
            return None;
        }
        if (index - visual).abs() > (VISIBLE_SLOTS / 2) as f64 + 0.5 {
            return None;
        }
        Some(index as usize)
    }

    pub(crate) fn is_over_panel(&self, point: Point<f64, Logical>) -> bool {
        self.visible && !self.apps.is_empty() && self.panel().contains(point)
    }

    pub(crate) fn update_hover(&mut self, point: Point<f64, Logical>) -> bool {
        // Leaving the dock keeps the last selection so the name capsule and
        // highlight stay put; only hovering another icon moves it.
        let Some(index) = self.index_at(point) else {
            return false;
        };
        if Some(index) == self.selected {
            return false;
        }
        self.position.retarget(
            index as f64,
            Duration::from_millis(MOVE_ANIM_MS),
            Easing::EaseOutCubic,
        );
        self.selected = Some(index);
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

        let (screen_w, screen_h) = (output_size.0.max(1), output_size.1.max(1));
        let logical_w = (screen_w as f64 / scale.x).max(1.0);
        let logical_h = (screen_h as f64 / scale.y).max(1.0);
        self.center_y = logical_h / 2.0;

        let panel = self.panel();
        let region = physical_region(&panel, scale, screen_w, screen_h);
        let tick = if self.position.is_animating() {
            self.position.frame_tick()
        } else {
            u64::MAX
        };
        let key = (self.apps.len(), self.selected, tick, region);

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
            let position = self.position.value();
            let selected = self.selected;
            if let Some(pixels) =
                render_pixels(region.2, region.3, scale, (region.0, region.1), |ctx| {
                    draw_scene(ctx, apps, &panel, position, selected, logical_w)
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

        let gradient_w =
            (logical_w * BACKDROP_WIDTH_RATIO).clamp(BACKDROP_WIDTH_MIN, BACKDROP_WIDTH_MAX);
        let backdrop_w = ((gradient_w * scale.x).ceil() as i32).clamp(1, screen_w);
        let backdrop_h = screen_h;
        let backdrop_size = (backdrop_w, backdrop_h);

        if self.backdrop_buffer.is_none() || self.backdrop_size != Some(backdrop_size) {
            if let Some(pixels) =
                render_backdrop_pixels(backdrop_w, backdrop_h, scale, gradient_w)
            {
                let mut buffer = MemoryRenderBuffer::new(
                    Fourcc::Argb8888,
                    (backdrop_w, backdrop_h),
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
                        Ok(buffer_damage(backdrop_w, backdrop_h))
                    });
                }
                self.backdrop_buffer = Some(buffer);
                self.backdrop_size = Some(backdrop_size);
            }
        }

        let mut elements = Vec::with_capacity(2);

        // Front: the docked launcher capsule, icons and label
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

        // Back: full-height dark gradient backdrop fading out to the right
        if let Some(backdrop_buffer) = self.backdrop_buffer.as_ref() {
            if let Ok(elem) = MemoryRenderBufferRenderElement::from_buffer(
                renderer,
                Point::<f64, Physical>::from((0.0, 0.0)),
                backdrop_buffer,
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
