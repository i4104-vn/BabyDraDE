use smithay::{
    backend::renderer::{
        element::{
            memory::MemoryRenderBufferRenderElement, render_elements,
            surface::WaylandSurfaceRenderElement, AsRenderElements,
        },
        glow::GlowRenderer,
    },
    desktop::Window,
    utils::{Logical, Point, Scale},
};

render_elements! {
    pub CustomRenderElements<=GlowRenderer>;
    Surface=WaylandSurfaceRenderElement<GlowRenderer>,
    Cursor=MemoryRenderBufferRenderElement<GlowRenderer>,
}

/// Renders native Wayland client surface elements (including CSD/GTK titlebar & decorations)
pub fn render_window(
    renderer: &mut GlowRenderer,
    window: &Window,
    location: Point<i32, Logical>,
    scale: Scale<f64>,
) -> Vec<CustomRenderElements> {
    window
        .render_elements::<WaylandSurfaceRenderElement<GlowRenderer>>(
            renderer,
            location.to_physical_precise_round(scale),
            scale,
            1.0,
        )
        .into_iter()
        .map(CustomRenderElements::Surface)
        .collect()
}
