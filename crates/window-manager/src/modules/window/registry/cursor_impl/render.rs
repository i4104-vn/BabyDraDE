use smithay::{
    backend::renderer::{
        element::{memory::MemoryRenderBufferRenderElement, Kind},
        glow::GlowRenderer,
    },
    utils::{Logical, Point, Scale},
};

use crate::modules::render::CustomRenderElements;

use super::models::Cursor;

impl Cursor {
    pub(crate) fn element(
        &self,
        renderer: &mut GlowRenderer,
        pos: Point<f64, Logical>,
        scale: Scale<f64>,
    ) -> Option<CustomRenderElements> {
        let cursor = self.cache.get(&self.current).unwrap_or(&self.fallback);

        // Align the image's hotspot (not its top-left corner) with the pointer.
        let phys_pos = pos.to_physical_precise_round(scale) - cursor.hotspot;
        MemoryRenderBufferRenderElement::from_buffer(
            renderer,
            phys_pos.to_f64(),
            &cursor.buffer,
            None,
            None,
            None,
            Kind::Cursor,
        )
        .ok()
        .map(CustomRenderElements::Memory)
    }
}
