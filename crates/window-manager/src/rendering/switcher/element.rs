use cairo::{Context, Format, ImageSurface};
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            element::{
                memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
                utils::{
                    constrain_as_render_elements, ConstrainAlign, ConstrainScaleBehavior,
                },
                Kind,
            },
            glow::GlowRenderer,
        },
    },
    desktop::Window,
    utils::{Buffer, Logical, Physical, Point, Rectangle, Scale, Size, Transform},
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

        let Some(layout) = compute_layout(switcher, width as f64, height as f64) else {
            return Vec::new();
        };

        let animation_active = switcher.transition_started.elapsed() < ANIMATION_DURATION;
        let render_key = (switcher.transition_started, switcher.selected_index);

        if animation_active || self.last_rendered != Some(render_key) {
            if let Some(bg_buf) = self.bg_buffer.as_mut() {
                if let Some(pixels) = render_pixels(width, height, |ctx| {
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
                if let Some(pixels) = render_pixels(width, height, |ctx| {
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

        // 2. Active Stage window preview
        let (front_idx, front_layout) = &layout.stage_front;
        if let Some(entry) = switcher.entries.get(*front_idx) {
            elements.extend(render_window_preview(
                renderer,
                &entry.window,
                front_layout.preview,
                scale,
                layout.alpha as f32,
            ));
        }

        // 3. Peek Stage window preview (if 3+ windows, peeking behind front window)
        if let Some((peek_idx, peek_layout)) = &layout.stage_peek {
            if let Some(entry) = switcher.entries.get(*peek_idx) {
                elements.extend(render_window_preview(
                    renderer,
                    &entry.window,
                    peek_layout.preview,
                    scale,
                    (0.85 * layout.alpha) as f32,
                ));
            }
        }

        // 4. Dock thumbnail window previews (left strip)
        for (dock_idx, dock_layout) in &layout.dock_items {
            if let Some(entry) = switcher.entries.get(*dock_idx) {
                elements.extend(render_window_preview(
                    renderer,
                    &entry.window,
                    dock_layout.preview,
                    scale,
                    (0.95 * layout.alpha) as f32,
                ));
            }
        }

        // 5. Background overlay (drop shadows, dark glass card backgrounds, ambient tint)
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

fn render_window_preview(
    renderer: &mut GlowRenderer,
    window: &Window,
    preview_rect: Rect,
    scale: Scale<f64>,
    alpha: f32,
) -> Vec<CustomRenderElements> {
    let geo = window.geometry();
    let bbox = window.bbox();
    let size = if geo.size.w > 0 && geo.size.h > 0 {
        geo.size
    } else if bbox.size.w > 0 && bbox.size.h > 0 {
        bbox.size
    } else {
        (800, 600).into()
    };

    let ref_size: Size<i32, Physical> = size.to_physical_precise_round(scale);
    let reference = Rectangle::new(
        (0, 0).into(),
        (ref_size.w.max(10), ref_size.h.max(10)).into(),
    );

    let loc = Point::<f64, Logical>::from((-geo.loc.x as f64, -geo.loc.y as f64));
    let loc_phys: Point<i32, Physical> = loc.to_physical_precise_round(scale);

    let constrain_loc: Point<i32, Physical> = Point::from((
        preview_rect.x.round() as i32,
        preview_rect.y.round() as i32,
    ));
    let constrain_size: Size<i32, Physical> = (
        (preview_rect.width.round() as i32).max(10),
        (preview_rect.height.round() as i32).max(10),
    )
        .into();
    let constrain = Rectangle::new(constrain_loc, constrain_size);

    constrain_as_render_elements::<GlowRenderer, Window, CustomRenderElements>(
        window,
        renderer,
        loc_phys,
        alpha,
        constrain,
        reference,
        ConstrainScaleBehavior::Fit,
        ConstrainAlign::CENTER,
        scale,
    )
    .collect()
}

fn render_pixels<F>(width: i32, height: i32, draw_fn: F) -> Option<Vec<u8>>
where
    F: FnOnce(&Context) -> Result<(), cairo::Error>,
{
    let mut surface = ImageSurface::create(Format::ARgb32, width, height).ok()?;
    {
        let context = Context::new(&surface).ok()?;
        draw_fn(&context).ok()?;
    }
    surface.flush();
    surface.data().ok().map(|data| data.to_vec())
}
