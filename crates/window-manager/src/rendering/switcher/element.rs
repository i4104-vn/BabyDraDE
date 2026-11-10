use cairo::{Context, Format, ImageSurface};
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            element::{
                memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
                Kind,
            },
            glow::GlowRenderer,
        },
    },
    utils::{Buffer, Physical, Point, Rectangle, Transform},
};

use crate::compositor::AltTabSwitcher;

use super::{
    super::CustomRenderElements,
    view::{draw_switcher, ANIMATION_DURATION},
};

#[derive(Default)]
pub struct SwitcherOverlay {
    buffer: Option<MemoryRenderBuffer>,
    size: (i32, i32),
    last_rendered: Option<(std::time::Instant, usize)>,
}

impl SwitcherOverlay {
    pub(crate) fn element(
        &mut self,
        renderer: &mut GlowRenderer,
        switcher: Option<&AltTabSwitcher>,
        size: (i32, i32),
    ) -> Option<CustomRenderElements> {
        let Some(switcher) = switcher else {
            self.last_rendered = None;
            return None;
        };
        let (width, height) = (size.0.max(1), size.1.max(1));
        if self.size != (width, height) {
            self.buffer = Some(MemoryRenderBuffer::new(
                Fourcc::Argb8888,
                (width, height),
                1,
                Transform::Normal,
                None,
            ));
            self.size = (width, height);
            self.last_rendered = None;
        }

        let buffer = self.buffer.as_mut()?;
        let animation_active = switcher.transition_started.elapsed() < ANIMATION_DURATION;
        let render_key = (switcher.transition_started, switcher.selected_index);
        if animation_active || self.last_rendered != Some(render_key) {
            let pixels = render_pixels(width, height, switcher)?;
            let mut context = buffer.render();
            let _ = context.draw(|buffer| {
                if buffer.len() != pixels.len() {
                    return Err(());
                }
                buffer.copy_from_slice(&pixels);
                Ok(vec![Rectangle::<i32, Buffer>::from_size(
                    (width, height).into(),
                )])
            });
            drop(context);
            self.last_rendered = Some(render_key);
        }

        MemoryRenderBufferRenderElement::from_buffer(
            renderer,
            Point::<f64, Physical>::from((0.0, 0.0)),
            buffer,
            None,
            None,
            None,
            Kind::Unspecified,
        )
        .ok()
        .map(CustomRenderElements::Memory)
    }
}

fn render_pixels(width: i32, height: i32, switcher: &AltTabSwitcher) -> Option<Vec<u8>> {
    let mut surface = ImageSurface::create(Format::ARgb32, width, height).ok()?;
    {
        let context = Context::new(&surface).ok()?;
        draw_switcher(&context, width as f64, height as f64, switcher).ok()?;
    }
    surface.flush();
    surface.data().ok().map(|data| data.to_vec())
}

