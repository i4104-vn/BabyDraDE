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

use crate::adapters::rendering::blur::BlurRenderElement;

#[derive(Debug, Clone)]
pub(crate) struct WindowBlurId(pub(crate) Id);

#[derive(Debug, Clone)]
pub(crate) struct LayerBlurId(pub(crate) Id);

pub(crate) type ConstrainedSurface = CropRenderElement<
    RelocateRenderElement<RescaleRenderElement<WaylandSurfaceRenderElement<GlowRenderer>>>,
>;
pub(crate) type DockClippedSurface = CropRenderElement<ConstrainedSurface>;

render_elements! {
    pub(crate) CustomRenderElements<=GlowRenderer>;
    Surface=WaylandSurfaceRenderElement<GlowRenderer>,
    Memory=MemoryRenderBufferRenderElement<GlowRenderer>,
    Blur=BlurRenderElement,
    Constrained=ConstrainedSurface,
    DockClipped=DockClippedSurface,
}
