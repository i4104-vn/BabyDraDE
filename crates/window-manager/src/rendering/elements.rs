use smithay::{
    backend::renderer::{
        element::{
            memory::MemoryRenderBufferRenderElement, render_elements,
            surface::WaylandSurfaceRenderElement, AsRenderElements,
        },
        glow::GlowRenderer,
    },
    desktop::{layer_map_for_output, Window},
    output::Output,
    utils::{Logical, Point, Scale},
    wayland::shell::wlr_layer::Layer,
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
    let render_loc = location - window.geometry().loc;
    window
        .render_elements::<WaylandSurfaceRenderElement<GlowRenderer>>(
            renderer,
            render_loc.to_physical_precise_round(scale),
            scale,
            1.0,
        )
        .into_iter()
        .map(CustomRenderElements::Surface)
        .collect()
}

/// Renders surfaces belonging to the specified WlrLayer layers (e.g. Top, Overlay, Background)
pub fn render_layer_surfaces(
    renderer: &mut GlowRenderer,
    output: &Output,
    scale: Scale<f64>,
    layers_filter: &[Layer],
) -> Vec<CustomRenderElements> {
    let layer_map = layer_map_for_output(output);
    let mut elements = Vec::new();
    for layer in layer_map.layers().rev() {
        if layers_filter.contains(&layer.layer()) {
            if let Some(geo) = layer_map.layer_geometry(layer) {
                let render_loc = geo.loc.to_physical_precise_round(scale);
                elements.extend(
                    layer
                        .render_elements::<WaylandSurfaceRenderElement<GlowRenderer>>(
                            renderer,
                            render_loc,
                            scale,
                            1.0,
                        )
                        .into_iter()
                        .map(CustomRenderElements::Surface),
                );
            }
        }
    }
    elements
}
