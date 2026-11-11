use smithay::backend::renderer::{
    element::{
        memory::MemoryRenderBufferRenderElement,
        render_elements,
        surface::WaylandSurfaceRenderElement,
        utils::{CropRenderElement, RelocateRenderElement, RescaleRenderElement},
        Id,
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
pub type DockClippedSurface = CropRenderElement<ConstrainedSurface>;

render_elements! {
    pub CustomRenderElements<=GlowRenderer>;
    Surface=WaylandSurfaceRenderElement<GlowRenderer>,
    Memory=MemoryRenderBufferRenderElement<GlowRenderer>,
    Blur=BlurRenderElement,
    Constrained=ConstrainedSurface,
    DockClipped=DockClippedSurface,
}
