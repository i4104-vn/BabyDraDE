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
    utils::{Buffer, Logical, Point, Rectangle, Scale, Transform},
};

use super::CustomRenderElements;

const CURSOR_WIDTH: i32 = 16;
const CURSOR_HEIGHT: i32 = 20;

const CURSOR_PIXELS: &[u8; 16 * 20] = &[
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
    1, 2, 2, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 1, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 0, 0, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub struct Cursor {
    buffer: MemoryRenderBuffer,
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

impl Cursor {
    pub fn new() -> Self {
        let mut buffer = MemoryRenderBuffer::new(
            Fourcc::Argb8888,
            (CURSOR_WIDTH, CURSOR_HEIGHT),
            1,
            Transform::Normal,
            None,
        );
        {
            let mut context = buffer.render();
            let _ = context.draw(|buf| {
                for (idx, &pixel_type) in CURSOR_PIXELS.iter().enumerate() {
                    let pixel = match pixel_type {
                        1 => [0, 0, 0, 255],           // Black outline
                        2 => [230, 248, 255, 255],     // Bright cyan-white body
                        _ => [0, 0, 0, 0],             // Transparent
                    };
                    buf[idx * 4..idx * 4 + 4].copy_from_slice(&pixel);
                }
                Ok::<_, ()>(vec![Rectangle::<i32, Buffer>::new(
                    (0, 0).into(),
                    (CURSOR_WIDTH, CURSOR_HEIGHT).into(),
                )])
            });
        }

        Self { buffer }
    }

    pub fn element(
        &self,
        renderer: &mut GlowRenderer,
        pos: Point<f64, Logical>,
        scale: Scale<f64>,
    ) -> Option<CustomRenderElements> {
        let phys_pos = pos.to_physical_precise_round(scale);
        MemoryRenderBufferRenderElement::from_buffer(
            renderer,
            phys_pos,
            &self.buffer,
            None,
            None,
            None,
            Kind::Cursor,
        )
        .ok()
        .map(CustomRenderElements::Memory)
    }
}
