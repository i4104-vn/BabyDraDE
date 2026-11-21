use std::time::Instant;

use cairo::{Context, Format, ImageSurface};
use smithay::{
    backend::renderer::element::utils::{ConstrainAlign, ConstrainScaleBehavior},
    backend::renderer::{
        element::{
            memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
            utils::CropRenderElement,
            Kind,
        },
        glow::GlowRenderer,
    },
    desktop::{
        space::{constrain_space_element, ConstrainBehavior, ConstrainReference},
        Window,
    },
    utils::{Buffer, Logical, Physical, Point, Rectangle, Scale, Size, Transform},
};

use crate::compositor::AltTabSwitcher;

use super::{
    draw::Rect,
    view::{compute_layout, draw_switcher_foreground, SwitcherLayout},
};

use crate::rendering::{elements::types::ConstrainedSurface, CustomRenderElements};

#[derive(Default)]
pub struct SwitcherOverlay {
    fg_buffer: Option<MemoryRenderBuffer>,
    /// Vùng vật lý mà `fg_buffer` bao phủ: `(x, y, width, height)`.
    /// Chỉ chứa dock thay vì toàn màn hình, nên mỗi frame animation chỉ phải
    /// cấp phát và copy vài trăm KB thay vì toàn bộ framebuffer.
    fg_region: Option<(i32, i32, i32, i32)>,
    last_rendered: Option<(Instant, usize, usize, u64)>,
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
        let logical_w = (width as f64 / scale.x).max(1.0);
        let logical_h = (height as f64 / scale.y).max(1.0);

        let Some(layout) = compute_layout(switcher, logical_w, logical_h) else {
            return Vec::new();
        };

        let region = dock_region(&layout, scale, width, height);
        if self.fg_region != Some(region) {
            self.fg_buffer = Some(MemoryRenderBuffer::new(
                smithay::backend::allocator::Fourcc::Argb8888,
                (region.2, region.3),
                1,
                Transform::Normal,
                None,
            ));
            self.fg_region = Some(region);
            self.last_rendered = None;
        }

        // Re-render the Cairo foreground while the selection tween is moving.
        // The microsecond tick gives the 120 fps loop a distinct frame every
        // time. This must also run with fewer than five apps: there the dock
        // items are static but the highlight frame glides between them, and
        // skipping these frames made it jump in one step.
        let animation_tick = if switcher.scroll.is_animating() {
            switcher.scroll.frame_tick()
        } else {
            u64::MAX
        };
        let render_key = (
            switcher.scroll.started_at(),
            switcher.selected_index,
            switcher.entries.len(),
            animation_tick,
        );

        if self.last_rendered != Some(render_key) {
            let (origin_x, origin_y, buffer_w, buffer_h) = region;
            if let Some(fg_buf) = self.fg_buffer.as_mut() {
                if let Some(pixels) =
                    render_pixels(buffer_w, buffer_h, scale, (origin_x, origin_y), |ctx| {
                        draw_switcher_foreground(ctx, switcher, &layout)
                    })
                {
                    let mut context = fg_buf.render();
                    let _ = context.draw(|buf| {
                        if buf.len() == pixels.len() {
                            buf.copy_from_slice(&pixels);
                            Ok(vec![Rectangle::<i32, Buffer>::from_size(
                                (buffer_w, buffer_h).into(),
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

        // 1. Static dock decorations, drawn above the window previews.
        if let (Some(fg_buf), Some((origin_x, origin_y, _, _))) =
            (self.fg_buffer.as_mut(), self.fg_region)
        {
            if let Ok(elem) = MemoryRenderBufferRenderElement::from_buffer(
                renderer,
                Point::<f64, Physical>::from((origin_x as f64, origin_y as f64)),
                fg_buf,
                None,
                None,
                None,
                Kind::Unspecified,
            ) {
                elements.push(CustomRenderElements::Memory(elem));
            }
        }

        // 2. Center window preview. Its size and opacity animate during selection.
        let (front_idx, front_layout) = &layout.stage;
        if let Some(entry) = switcher.entries.get(*front_idx) {
            elements.extend(
                render_window_preview(
                    renderer,
                    &entry.window,
                    front_layout.preview,
                    scale,
                    front_layout.opacity as f32,
                    ConstrainScaleBehavior::Stretch,
                    ConstrainReference::Geometry,
                )
                .into_iter()
                .map(CustomRenderElements::Constrained),
            );
        }

        // 3. Dock thumbnails stay stationary with their aspect ratios preserved.
        let dock_crop = Rectangle::<i32, Physical>::new(
            Point::<i32, Physical>::from((
                (layout.dock_viewport.x * scale.x).floor() as i32,
                (layout.dock_viewport.y * scale.y).floor() as i32,
            )),
            Size::<i32, Physical>::from((
                ((layout.dock_viewport.x + layout.dock_viewport.width) * scale.x).ceil() as i32
                    - (layout.dock_viewport.x * scale.x).floor() as i32,
                ((layout.dock_viewport.y + layout.dock_viewport.height) * scale.y).ceil() as i32
                    - (layout.dock_viewport.y * scale.y).floor() as i32,
            )),
        );

        for (dock_idx, dock_layout) in &layout.dock_items {
            if let Some(entry) = switcher.entries.get(*dock_idx) {
                elements.extend(
                    render_window_preview(
                        renderer,
                        &entry.window,
                        dock_layout.preview,
                        scale,
                        (0.95 * dock_layout.opacity) as f32,
                        ConstrainScaleBehavior::Zoom,
                        ConstrainReference::Geometry,
                    )
                    .into_iter()
                    .filter_map(|preview| {
                        CropRenderElement::from_element(preview, scale, dock_crop)
                            .map(CustomRenderElements::DockClipped)
                    }),
                );
            }
        }

        elements
    }
}

/// Render a window scaled into `target_rect` (logical coordinates).
///
/// `scale_behavior` controls whether the window is stretched to fill the slot
/// (`Stretch`, used for the center stage) or zoomed to cover it (`Zoom`, used
/// for dock thumbnails so the dark card behind never shows at their edges).
///
/// Uses smithay's `constrain_space_element` pipeline so that the compositor's damage
/// tracker sees the correct physical geometry of every sub-surface element.
fn render_window_preview(
    renderer: &mut GlowRenderer,
    window: &Window,
    target_rect: Rect,
    output_scale: Scale<f64>,
    alpha: f32,
    scale_behavior: ConstrainScaleBehavior,
    reference: ConstrainReference,
) -> Vec<ConstrainedSurface> {
    // Guard: window must have a valid bounding-box so constrain can compute scale.
    let bbox = window.bbox();
    if bbox.size.w <= 0 || bbox.size.h <= 0 {
        // Try geometry as fallback.
        let geo = window.geometry();
        if geo.size.w <= 0 || geo.size.h <= 0 {
            return Vec::new();
        }
    }

    // The constrain rect in logical coordinates (what the layout computed).
    let constrain = Rectangle::<i32, Logical>::new(
        Point::<i32, Logical>::from((target_rect.x.round() as i32, target_rect.y.round() as i32)),
        Size::<i32, Logical>::from((
            (target_rect.width.round() as i32).max(4),
            (target_rect.height.round() as i32).max(4),
        )),
    );

    // `location` anchors the chosen reference rectangle (window geometry, not
    // the bbox with client-side shadows) at the top-left of the slot. Smithay
    // then scales/crops it into `constrain`, so CSD drop shadows outside the
    // geometry never become a dark rim around the preview.
    let location: Point<i32, Logical> = constrain.loc;

    let behavior = ConstrainBehavior {
        reference,
        behavior: scale_behavior,
        align: ConstrainAlign::CENTER,
    };

    constrain_space_element::<GlowRenderer, Window, ConstrainedSurface>(
        renderer,
        window,
        location,
        alpha,
        output_scale,
        constrain,
        behavior,
    )
    .collect()
}

/// Vùng vật lý của dock, làm tròn ra ngoài để không cắt mất cạnh khử răng cưa.
fn dock_region(
    layout: &SwitcherLayout,
    scale: Scale<f64>,
    screen_w: i32,
    screen_h: i32,
) -> (i32, i32, i32, i32) {
    let vp = layout.dock_viewport;
    let x0 = (vp.x * scale.x).floor().clamp(0.0, screen_w as f64) as i32;
    let y0 = (vp.y * scale.y).floor().clamp(0.0, screen_h as f64) as i32;
    let x1 = ((vp.x + vp.width) * scale.x)
        .ceil()
        .clamp(0.0, screen_w as f64) as i32;
    let y1 = ((vp.y + vp.height) * scale.y)
        .ceil()
        .clamp(0.0, screen_h as f64) as i32;

    (x0, y0, (x1 - x0).max(1), (y1 - y0).max(1))
}

fn render_pixels<F>(
    width: i32,
    height: i32,
    scale: Scale<f64>,
    origin: (i32, i32),
    draw_fn: F,
) -> Option<Vec<u8>>
where
    F: FnOnce(&Context) -> Result<(), cairo::Error>,
{
    let mut surface = ImageSurface::create(Format::ARgb32, width, height).ok()?;
    {
        let context = Context::new(&surface).ok()?;
        context.scale(scale.x, scale.y);
        // Buffer chỉ phủ vùng dock: dịch hệ trục để phần vẽ vẫn dùng toạ độ
        // logic của toàn màn hình.
        context.translate(-(origin.0 as f64) / scale.x, -(origin.1 as f64) / scale.y);
        draw_fn(&context).ok()?;
    }
    surface.flush();
    surface.data().ok().map(|data| data.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::switcher::view::CardLayout;

    fn test_layout(viewport: Rect) -> SwitcherLayout {
        SwitcherLayout {
            stage: (
                0,
                CardLayout {
                    card: viewport,
                    preview: viewport,
                    radius: 0.0,
                    opacity: 1.0,
                },
            ),
            dock_viewport: viewport,
            dock_items: Vec::new(),
            dock_target: viewport,
        }
    }

    #[test]
    fn dock_region_rounds_outward_and_clamps_to_screen() {
        let layout = test_layout(Rect::new(24.0, 32.0, 175.0, 1016.0));

        assert_eq!(
            dock_region(&layout, Scale::from(2.0), 3840, 2160),
            (48, 64, 350, 2032)
        );
        assert_eq!(
            dock_region(&layout, Scale::from(2.0), 300, 1000),
            (48, 64, 252, 936)
        );
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
