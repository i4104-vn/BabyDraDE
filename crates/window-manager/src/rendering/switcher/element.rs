use cairo::{Context, Format, ImageSurface};
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            element::{
                memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
                surface::WaylandSurfaceRenderElement,
                AsRenderElements, Kind,
            },
            glow::GlowRenderer,
        },
    },
    desktop::Window,
    utils::{Buffer, Physical, Point, Rectangle, Scale, Transform},
};

use crate::compositor::AltTabSwitcher;

use super::{
    super::CustomRenderElements,
    draw::Rect,
    view::{
        compute_layout, draw_switcher_background, draw_switcher_foreground, ANIMATION_DURATION,
    },
};

#[derive(Default)]
pub struct SwitcherOverlay {
    bg_buffer: Option<MemoryRenderBuffer>,
    fg_buffer: Option<MemoryRenderBuffer>,
    size: (i32, i32),
    last_rendered: Option<(std::time::Instant, usize)>,
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
            return Vec::new();
        };

        if switcher.entries.is_empty() {
            return Vec::new();
        }

        let (width, height) = (size.0.max(1), size.1.max(1));
        if self.size != (width, height) {
            self.bg_buffer = Some(MemoryRenderBuffer::new(
                Fourcc::Argb8888,
                (width, height),
                1,
                Transform::Normal,
                None,
            ));
            self.fg_buffer = Some(MemoryRenderBuffer::new(
                Fourcc::Argb8888,
                (width, height),
                1,
                Transform::Normal,
                None,
            ));
            self.size = (width, height);
            self.last_rendered = None;
        }

        let logical_w = (width as f64 / scale.x).max(1.0);
        let logical_h = (height as f64 / scale.y).max(1.0);

        let Some(layout) = compute_layout(switcher, logical_w, logical_h) else {
            return Vec::new();
        };

        let animation_active = switcher.transition_started.elapsed() < ANIMATION_DURATION;
        let render_key = (switcher.transition_started, switcher.selected_index);

        if animation_active || self.last_rendered != Some(render_key) {
            if let Some(bg_buf) = self.bg_buffer.as_mut() {
                if let Some(pixels) = render_pixels(width, height, scale, |ctx| {
                    draw_switcher_background(ctx, switcher, &layout)
                }) {
                    let mut context = bg_buf.render();
                    let _ = context.draw(|buf| {
                        if buf.len() == pixels.len() {
                            buf.copy_from_slice(&pixels);
                            Ok(vec![Rectangle::<i32, Buffer>::from_size(
                                (width, height).into(),
                            )])
                        } else {
                            Err(())
                        }
                    });
                }
            }

            if let Some(fg_buf) = self.fg_buffer.as_mut() {
                if let Some(pixels) = render_pixels(width, height, scale, |ctx| {
                    draw_switcher_foreground(ctx, switcher, &layout)
                }) {
                    let mut context = fg_buf.render();
                    let _ = context.draw(|buf| {
                        if buf.len() == pixels.len() {
                            buf.copy_from_slice(&pixels);
                            Ok(vec![Rectangle::<i32, Buffer>::from_size(
                                (width, height).into(),
                            )])
                        } else {
                            Err(())
                        }
                    });
                }
            }

            self.last_rendered = Some(render_key);
        }

        let mut elements = Vec::new();

        // 1. Foreground overlay (borders, three dots header, title pill, dock app badges)
        if let Some(fg_buf) = self.fg_buffer.as_mut() {
            if let Ok(elem) = MemoryRenderBufferRenderElement::from_buffer(
                renderer,
                Point::<f64, Physical>::from((0.0, 0.0)),
                fg_buf,
                None,
                None,
                None,
                Kind::Unspecified,
            ) {
                elements.push(CustomRenderElements::Memory(elem));
            }
        }

        // 2. Active Stage window (ONLY 1 REAL APP in center, rendered directly with native Wayland surfaces!)
        let (front_idx, front_layout) = &layout.stage;
        if let Some(entry) = switcher.entries.get(*front_idx) {
            elements.extend(render_scaled_window(
                renderer,
                &entry.window,
                front_layout.preview,
                scale,
                layout.alpha as f32,
            ));
        }

        // 3. Dock thumbnail windows (ALL existing apps on left strip, rendered directly with native Wayland surfaces!)
        for (dock_idx, dock_layout) in &layout.dock_items {
            if let Some(entry) = switcher.entries.get(*dock_idx) {
                elements.extend(render_scaled_window(
                    renderer,
                    &entry.window,
                    dock_layout.preview,
                    scale,
                    (0.95 * layout.alpha) as f32,
                ));
            }
        }

        // 4. Background overlay (drop shadows under windows)
        if let Some(bg_buf) = self.bg_buffer.as_mut() {
            if let Ok(elem) = MemoryRenderBufferRenderElement::from_buffer(
                renderer,
                Point::<f64, Physical>::from((0.0, 0.0)),
                bg_buf,
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

    #[allow(dead_code)]
    pub(crate) fn element(
        &mut self,
        renderer: &mut GlowRenderer,
        switcher: Option<&AltTabSwitcher>,
        size: (i32, i32),
    ) -> Option<CustomRenderElements> {
        self.render(renderer, switcher, size, Scale::from(1.0))
            .into_iter()
            .next()
    }
}

fn render_scaled_window(
    renderer: &mut GlowRenderer,
    window: &Window,
    target_rect: Rect,
    output_scale: Scale<f64>,
    alpha: f32,
) -> Vec<CustomRenderElements> {
    let geo = window.geometry();
    let bbox = window.bbox();
    let win_size = if geo.size.w > 0 && geo.size.h > 0 {
        geo.size
    } else if bbox.size.w > 0 && bbox.size.h > 0 {
        bbox.size
    } else {
        return Vec::new();
    };

    let scale_x = target_rect.width / win_size.w as f64;
    let scale_y = target_rect.height / win_size.h as f64;
    let fit_scale = scale_x.min(scale_y);
    let scaled_w = win_size.w as f64 * fit_scale;
    let scaled_h = win_size.h as f64 * fit_scale;

    let fit_x = target_rect.x + (target_rect.width - scaled_w) / 2.0;
    let fit_y = target_rect.y + (target_rect.height - scaled_h) / 2.0;

    let eff_scale = Scale::from((
        output_scale.x * fit_scale,
        output_scale.y * fit_scale,
    ));

    let root_loc_x = fit_x - geo.loc.x as f64 * fit_scale;
    let root_loc_y = fit_y - geo.loc.y as f64 * fit_scale;
    let phys_loc: Point<i32, Physical> = Point::from((
        (root_loc_x * output_scale.x).round() as i32,
        (root_loc_y * output_scale.y).round() as i32,
    ));

    window
        .render_elements::<WaylandSurfaceRenderElement<GlowRenderer>>(
            renderer,
            phys_loc,
            eff_scale,
            alpha,
        )
        .into_iter()
        .map(CustomRenderElements::Surface)
        .collect()
}

fn render_pixels<F>(
    width: i32,
    height: i32,
    scale: Scale<f64>,
    draw_fn: F,
) -> Option<Vec<u8>>
where
    F: FnOnce(&Context) -> Result<(), cairo::Error>,
{
    let mut surface = ImageSurface::create(Format::ARgb32, width, height).ok()?;
    {
        let context = Context::new(&surface).ok()?;
        context.scale(scale.x, scale.y);
        draw_fn(&context).ok()?;
    }
    surface.flush();
    surface.data().ok().map(|data| data.to_vec())
}
