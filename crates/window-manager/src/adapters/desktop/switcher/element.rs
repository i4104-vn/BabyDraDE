//! Composes the switcher overlay out of its foreground, preview and backdrop layers.

mod backdrop;
mod foreground;
mod preview;

use smithay::{
    backend::renderer::{element::memory::MemoryRenderBuffer, glow::GlowRenderer},
    utils::Scale,
};

use crate::adapters::desktop::switcher::{view::compute_layout, AltTabSwitcher};
use crate::adapters::rendering::CustomRenderElements;

#[derive(Default)]
pub(crate) struct SwitcherOverlay {
    fg_buffer: Option<MemoryRenderBuffer>,
    /// Physical region covered by `fg_buffer`: `(x, y, width, height)`.
    /// It only holds the dock rather than the whole screen, so each animation
    /// frame allocates and copies a few hundred KB instead of the full framebuffer.
    fg_region: Option<(i32, i32, i32, i32)>,
    last_rendered: Option<foreground::RenderKey>,
    backdrop_buffer: Option<MemoryRenderBuffer>,
    backdrop_size: Option<(i32, i32)>,
    backdrop_tick: Option<u64>,
}

impl SwitcherOverlay {
    pub(crate) fn render(
        &mut self,
        renderer: &mut GlowRenderer,
        switcher: Option<&AltTabSwitcher>,
        size: (i32, i32),
        scale: Scale<f64>,
    ) -> Vec<CustomRenderElements> {
        let Some(switcher) = switcher else {
            self.last_rendered = None;
            self.backdrop_tick = None;
            return Vec::new();
        };

        if switcher.entries.is_empty() {
            return Vec::new();
        }

        let (width, height) = (size.0.max(1), size.1.max(1));
        let logical_w = (width as f64 / scale.x).max(1.0);
        let logical_h = (height as f64 / scale.y).max(1.0);

        let Some(layout) = compute_layout(switcher, logical_w, logical_h) else {
            return Vec::new();
        };

        let region = foreground::overlay_region(&layout, scale, width, height);

        let animation_tick = if switcher.scroll.is_animating() {
            switcher.scroll.frame_tick()
        } else {
            u64::MAX
        };
        let stage_tick = if switcher.stage.is_animating() {
            switcher.stage.frame_tick()
        } else {
            u64::MAX
        };
        let highlight_tick = if switcher.highlight.is_animating() {
            switcher.highlight.frame_tick()
        } else {
            u64::MAX
        };
        let presence = switcher.presence();
        let presence_tick = if switcher.presence.is_animating() {
            switcher.presence.frame_tick()
        } else {
            u64::MAX
        };
        // Slide out to the right when closed, dock at the screen edge when open.
        let slide_x = (1.0 - presence) * common::constants::launcher::PANEL_WIDTH;
        let render_key = (
            switcher.scroll.started_at(),
            switcher.selected_index,
            switcher.hovered,
            switcher.entries.len(),
            animation_tick,
            stage_tick,
            presence_tick,
            highlight_tick,
        );

        foreground::update(
            self, switcher, &layout, region, scale, slide_x, presence, render_key,
        );
        let backdrop_w = backdrop::update(
            self,
            logical_w,
            width,
            height,
            scale,
            presence,
            presence_tick,
        );

        let mut elements = Vec::new();
        // 1. Static dock decorations, drawn above the window previews.
        elements.extend(foreground::elements(self, renderer));
        // 2. One small live preview beside the active icon.
        elements.extend(preview::elements(
            renderer, switcher, &layout, slide_x, scale,
        ));
        // 3. Backdrop behind everything else (index 0 is topmost).
        elements.extend(backdrop::elements(self, backdrop_w, width, renderer));

        elements
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::Rect;
    use super::foreground::overlay_region;
    use crate::adapters::desktop::switcher::view::SwitcherLayout;
    use crate::adapters::rendering::buffer::render_pixels;
    use smithay::utils::Scale;

    fn test_layout(panel: Rect, preview: Rect) -> SwitcherLayout {
        SwitcherLayout {
            panel: ui_kit::components::dock::DockLayout {
                x: panel.x,
                y: panel.y,
                width: panel.width,
                height: panel.height,
                center_y: panel.y + panel.height / 2.0,
            },
            selected: 0,
            preview,
            preview_alpha: 1.0,
        }
    }

    #[test]
    fn overlay_region_covers_capsule_and_preview() {
        let layout = test_layout(
            Rect::new(1848.0, 32.0, 72.0, 1016.0),
            Rect::new(1534.0, 400.0, 300.0, 176.0),
        );

        let region = overlay_region(&layout, Scale::from(1.0), 1920, 1080);
        assert_eq!(region, (1510, 8, 410, 1064));
    }

    #[test]
    fn overlay_region_clamps_to_screen() {
        let layout = test_layout(
            Rect::new(24.0, 32.0, 72.0, 1016.0),
            Rect::new(8.0, 400.0, 300.0, 176.0),
        );
        let region = overlay_region(&layout, Scale::from(1.0), 300, 1000);
        assert_eq!(region.0, 0);
        assert!(region.2 <= 300);
        assert!(region.3 <= 1000);
    }

    #[test]
    fn dock_buffer_maps_logical_coordinates_to_the_region_origin() {
        let pixels = render_pixels(8, 8, Scale::from(1.0), (4, 6), |ctx| {
            ctx.set_source_rgba(1.0, 0.0, 0.0, 1.0);
            ctx.rectangle(5.0, 7.0, 2.0, 2.0);
            ctx.fill()
        })
        .expect("cairo surface");

        // ARGB32 little-endian: byte 3 of each pixel is alpha. Device pixel
        // (0,0) maps to logical (4,6), outside the red rect; (1,1) is inside.
        assert_eq!(pixels[3], 0);
        assert_eq!(pixels[(8 + 1) * 4 + 3], 255);
    }
}
