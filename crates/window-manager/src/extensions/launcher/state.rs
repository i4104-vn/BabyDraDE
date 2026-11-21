//! Launcher state: app list, selection, pointer/wheel/keyboard input and the
//! render entry point. Drawing and geometry live in sibling modules.

use animation::{
    Easing, Tween, FADE_IN_DURATION, FADE_OUT_DURATION, NAVIGATION_DURATION, SCROLL_DURATION,
};
use common::{
    KEY_REPEAT_DELAY_MS as REPEAT_DELAY_MS, KEY_REPEAT_INTERVAL_MS as REPEAT_INTERVAL_MS,
};
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
use std::time::{Duration, Instant};

use crate::rendering::CustomRenderElements;

use super::buffer::{buffer_damage, physical_region, render_backdrop_pixels, render_pixels};
use super::draw::draw_scene;
use super::layout::{visual_center, Panel};
use super::{
    BackdropKey, BufferRegion, CacheKey, LauncherApp, BACKDROP_WIDTH_MIN, BACKDROP_WIDTH_RATIO,
    PANEL_WIDTH, SLOT, VISIBLE_SLOTS,
};

/// State of the launcher.
pub(crate) struct LauncherState {
    apps: Vec<LauncherApp>,
    visible: bool,
    /// Continuous selection position (index, fractional while animating).
    position: Tween,
    /// 0 while closed, 1 while open; drives the fade animation.
    presence: Tween,
    /// Still rendered while the close animation plays out.
    closing: bool,
    selected: Option<usize>,
    /// Vertical centre of the capsule in logical coordinates.
    center_y: f64,
    buffer: Option<MemoryRenderBuffer>,
    region: Option<BufferRegion>,
    cache_key: Option<CacheKey>,
    backdrop_buffer: Option<MemoryRenderBuffer>,
    backdrop_size: Option<(i32, i32)>,
    backdrop_cache_key: Option<BackdropKey>,
    /// Held arrow direction and timers used for auto-repeat.
    held_direction: Option<i32>,
    repeat_started: Option<Instant>,
    last_repeat: Option<Instant>,
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
            presence: Tween::settled(0.0),
            closing: false,
            selected: None,
            center_y: 0.0,
            buffer: None,
            region: None,
            cache_key: None,
            backdrop_buffer: None,
            backdrop_size: None,
            backdrop_cache_key: None,
            held_direction: None,
            repeat_started: None,
            last_repeat: None,
        }
    }

    fn panel(&self) -> Panel {
        Panel::new(self.apps.len(), self.center_y)
    }

    /// Horizontal offset of the capsule during the open/close animation:
    /// fully off-screen to the left when closed, docked at `PANEL_X` when open.
    fn slide_offset(&self) -> f64 {
        slide_offset_for(self.presence.value())
    }

    pub(crate) fn is_visible(&self) -> bool {
        self.visible && !self.closing
    }

    /// Toggles between opening and starting the close animation. Toggling
    /// again mid-close reopens the launcher.
    pub(crate) fn toggle(&mut self) -> bool {
        if self.visible && !self.closing {
            self.close();
        } else {
            self.open();
        }
        self.visible
    }

    fn open(&mut self) {
        self.visible = true;
        self.closing = false;
        let selected = self
            .selected
            .unwrap_or(0)
            .min(self.apps.len().saturating_sub(1));
        self.selected = Some(selected);
        self.position = Tween::settled(selected as f64);
        self.presence
            .retarget(1.0, FADE_IN_DURATION, Easing::EaseInOutCubic);
        self.cache_key = None;
    }

    /// Starts the close animation; `render` finalises it when the fade ends.
    pub(crate) fn close(&mut self) {
        if !self.visible {
            return;
        }
        self.closing = true;
        self.stop_key_repeat();
        self.presence
            .retarget(0.0, FADE_OUT_DURATION, Easing::EaseInOutCubic);
        self.cache_key = None;
    }

    fn finish_close(&mut self) {
        self.visible = false;
        self.closing = false;
        self.selected = None;
        self.position = Tween::settled(0.0);
        self.presence = Tween::settled(0.0);
        self.cache_key = None;
        self.backdrop_cache_key = None;
    }

    /// First press of an arrow key starts repeat tracking; backend-generated
    /// auto-repeat presses are ignored because `tick_key_repeat` drives them.
    /// Returns `true` only for the initial press.
    pub(crate) fn arrow_pressed(&mut self, direction: i32) -> bool {
        if !self.visible || self.closing || self.held_direction == Some(direction) {
            return false;
        }
        self.held_direction = Some(direction);
        let now = Instant::now();
        self.repeat_started = Some(now);
        self.last_repeat = Some(now);
        true
    }

    pub(crate) fn arrow_released(&mut self, direction: i32) {
        if self.held_direction == Some(direction) {
            self.stop_key_repeat();
        }
    }

    fn stop_key_repeat(&mut self) {
        self.held_direction = None;
        self.repeat_started = None;
        self.last_repeat = None;
    }

    /// Called once per frame to implement hold-to-navigate independently of
    /// whether the input backend generates auto-repeat events.
    fn tick_key_repeat(&mut self) {
        let Some(direction) = self.held_direction else {
            return;
        };
        let Some(started) = self.repeat_started else {
            return;
        };
        let now = Instant::now();
        if repeat_due(started, self.last_repeat.unwrap_or(started), now) {
            self.last_repeat = Some(now);
            self.move_selection(direction);
        }
    }

    /// Moves the keyboard selection with a smooth transition.
    pub(crate) fn move_selection(&mut self, delta: i32) {
        if !self.visible || self.closing || self.apps.is_empty() {
            return;
        }
        let current = self.selected.unwrap_or(0) as i32;
        let next = (current + delta).clamp(0, self.apps.len() as i32 - 1) as usize;
        if next as i32 == current {
            return;
        }
        self.position
            .retarget(next as f64, NAVIGATION_DURATION, Easing::EaseInOutCubic);
        self.selected = Some(next);
        self.cache_key = None;
    }

    pub(crate) fn activate(&mut self) {
        if !self.closing {
            if let Some(index) = self.selected {
                self.launch(index);
            }
        }
    }

    pub(crate) fn scroll_by(&mut self, slots: f64) {
        if self.closing || self.apps.is_empty() {
            return;
        }
        let current = self.position.target();
        let next = (current + slots)
            .round()
            .clamp(0.0, self.apps.len() as f64 - 1.0);
        if (next - current).abs() > f64::EPSILON {
            self.selected = Some(next as usize);
            self.position
                .retarget(next, SCROLL_DURATION, Easing::EaseInOutCubic);
            self.cache_key = None;
        }
    }

    pub(crate) fn index_at(&self, point: Point<f64, Logical>) -> Option<usize> {
        if !self.visible || self.closing || self.apps.is_empty() {
            return None;
        }
        let mut panel = self.panel();
        panel.x += self.slide_offset();
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
        if !self.visible || self.closing || self.apps.is_empty() {
            return false;
        }
        let mut panel = self.panel();
        panel.x += self.slide_offset();
        panel.contains(point)
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
        self.position
            .retarget(index as f64, NAVIGATION_DURATION, Easing::EaseInOutCubic);
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

        self.tick_key_repeat();
        if self.closing && !self.presence.is_animating() {
            self.finish_close();
            return Vec::new();
        }

        let (screen_w, screen_h) = (output_size.0.max(1), output_size.1.max(1));
        let logical_w = (screen_w as f64 / scale.x).max(1.0);
        let logical_h = (screen_h as f64 / scale.y).max(1.0);
        self.center_y = logical_h / 2.0;

        let panel = self.panel();
        let region = physical_region(&panel, scale, screen_w, screen_h);
        let presence_animating = self.presence.is_animating();
        let position_animating = self.position.is_animating();
        // Microsecond ticks (not frame indices) guarantee a fresh buffer on
        // every render, even when the event loop runs faster than 120 fps.
        let presence_tick = if presence_animating {
            self.presence.frame_tick()
        } else {
            u64::MAX
        };
        let tick = match (presence_animating, position_animating) {
            (true, true) => presence_tick.max(self.position.frame_tick()),
            (true, false) => presence_tick,
            (false, true) => self.position.frame_tick(),
            (false, false) => u64::MAX,
        };
        let presence = self.presence.value();
        let slide_x = slide_offset_for(presence);
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
                    draw_scene(
                        ctx, apps, &panel, position, selected, logical_w, presence, slide_x,
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
        let backdrop_h = screen_h;
        let backdrop_size = (backdrop_w, backdrop_h);

        let backdrop_key = (backdrop_size, presence_tick);
        if self.backdrop_buffer.is_none()
            || self.backdrop_size != Some(backdrop_size)
            || self.backdrop_cache_key != Some(backdrop_key)
        {
            if let Some(pixels) =
                render_backdrop_pixels(backdrop_w, backdrop_h, scale, gradient_w, presence)
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
                self.backdrop_cache_key = Some(backdrop_key);
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

/// Slide distance for a given presence value (0 = hidden left, 1 = docked).
fn slide_offset_for(presence: f64) -> f64 {
    -(1.0 - presence.clamp(0.0, 1.0)) * PANEL_WIDTH
}

/// Whether a held arrow should step again at `now`.
fn repeat_due(started: Instant, last: Instant, now: Instant) -> bool {
    now.duration_since(started) >= Duration::from_millis(REPEAT_DELAY_MS)
        && now.duration_since(last) >= Duration::from_millis(REPEAT_INTERVAL_MS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capsule_slides_in_from_the_left() {
        assert_eq!(slide_offset_for(0.0), -PANEL_WIDTH);
        assert_eq!(slide_offset_for(1.0), 0.0);
        assert_eq!(slide_offset_for(0.5), -PANEL_WIDTH / 2.0);
    }

    #[test]
    fn repeat_waits_for_delay_then_runs_at_interval() {
        let started = Instant::now();
        let now = started + Duration::from_millis(REPEAT_DELAY_MS - 1);
        assert!(!repeat_due(started, started, now));

        let now = started + Duration::from_millis(REPEAT_DELAY_MS);
        assert!(repeat_due(started, started, now));

        // After a repeat, the interval — not the initial delay — gates the
        // next step.
        let last = started + Duration::from_millis(REPEAT_DELAY_MS);
        assert!(!repeat_due(
            started,
            last,
            last + Duration::from_millis(REPEAT_INTERVAL_MS - 1)
        ));
        assert!(repeat_due(
            started,
            last,
            last + Duration::from_millis(REPEAT_INTERVAL_MS)
        ));
    }
}
