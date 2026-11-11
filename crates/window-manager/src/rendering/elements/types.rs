use smithay::backend::renderer::{
    element::{
        memory::MemoryRenderBufferRenderElement, render_elements,
        surface::WaylandSurfaceRenderElement, Id,
        utils::{CropRenderElement, RelocateRenderElement, RescaleRenderElement},
    },
    glow::GlowRenderer,
};

use crate::rendering::blur::BlurRenderElement;

#[derive(Debug, Clone)]
pub(crate) struct WindowBlurId(pub(crate) Id);

#[derive(Debug, Clone)]
pub(crate) struct LayerBlurId(pub(crate) Id);

pub type ConstrainedSurface = CropRenderElement<
    RelocateRenderElement<RescaleRenderElement<WaylandSurfaceRenderElement<GlowRenderer>>>,
>;

render_elements! {
    pub CustomRenderElements<=GlowRenderer>;
    Surface=WaylandSurfaceRenderElement<GlowRenderer>,
    Memory=MemoryRenderBufferRenderElement<GlowRenderer>,
    Blur=BlurRenderElement,
    Constrained=ConstrainedSurface,
}

