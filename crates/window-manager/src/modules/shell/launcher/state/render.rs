//! Compositor render elements for the launcher and search overlay.

use smithay::{
    backend::{allocator::Fourcc, renderer::element::memory::MemoryRenderBuffer},
    utils::{Scale, Transform},
};

use super::context::SurfaceContext;
use super::LauncherState;
use crate::modules::shell::launcher::backdrop::{
    buffer_damage, physical_region, render_backdrop_pixels,
};
use crate::modules::shell::launcher::render::draw_scene;
use crate::modules::shell::launcher::search::buffer::panel_region;
use crate::modules::shell::launcher::search::layout::SearchLayout;
use crate::modules::shell::launcher::{BACKDROP_WIDTH_MIN, BACKDROP_WIDTH_RATIO};
use crate::modules::render::buffer::render_pixels;
use crate::modules::render::CustomRenderElements;

impl LauncherState {
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

        // GTK may have edited the buffer since the last frame. Reconcile once
        // here so results/animations stay in step with what the user typed.
        let apps = &self.apps;
        self.search.sync(apps);

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
        let presence_tick = self.dock.presence.cache_tick();
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

        let layout = SearchLayout::compute(logical_w, logical_h, self.search.content_height());
        self.search.set_layout(layout);
        let search_region = panel_region(&layout, scale, screen_w, screen_h);

        let mut ctx = SurfaceContext {
            renderer,
            scale,
            screen: (screen_w, screen_h),
            logical: (logical_w, logical_h),
            search_presence,
            dock_presence,
            search_tick,
            search_region,
            region,
        };
        super::surface::render_search(self, &ctx);
        super::surface::collect_elements(self, &mut ctx)
    }
}
