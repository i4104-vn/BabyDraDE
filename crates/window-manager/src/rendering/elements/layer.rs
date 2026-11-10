use std::sync::{Arc, Mutex};

use smithay::{
    backend::renderer::{
        element::{surface::WaylandSurfaceRenderElement, AsRenderElements, Id},
        glow::GlowRenderer,
    },
    desktop::layer_map_for_output,
    output::Output,
    utils::{Rectangle, Scale},
    wayland::shell::wlr_layer::Layer,
};

use crate::rendering::blur::{BlurEngine, BlurRenderElement};

use super::types::{CustomRenderElements, LayerBlurId};

/// Renders surfaces belonging to the specified WlrLayer layers (e.g. Top, Overlay, Background)
/// and optionally attaches a blur backdrop for matching layers.
pub fn render_layer_surfaces(
    renderer: &mut GlowRenderer,
    output: &Output,
    scale: Scale<f64>,
    layers_filter: &[Layer],
    blur_engine: Option<&Arc<Mutex<BlurEngine>>>,
) -> Vec<CustomRenderElements> {
    let layer_map = layer_map_for_output(output);
    let mut elements = Vec::new();
    for layer in layer_map.layers().rev() {
        if layers_filter.contains(&layer.layer()) {
            if let Some(geo) = layer_map.layer_geometry(layer) {
                let render_loc = geo.loc.to_physical_precise_round(scale);

                // The desktop background and topbar are never blur targets.
                let should_blur = if layer.layer() == Layer::Background || layer.namespace().contains("topbar") {
                    false
                } else if let Some(engine_lock) = blur_engine {
                    if let Ok(engine) = engine_lock.lock() {
                        engine.matches_layer(layer.namespace())
                    } else {
                        false
                    }
                } else {
                    false
                };

                let layer_surfaces: Vec<CustomRenderElements> = layer
                    .render_elements::<WaylandSurfaceRenderElement<GlowRenderer>>(
                        renderer, render_loc, scale, 1.0,
                    )
                    .into_iter()
                    .map(CustomRenderElements::Surface)
                    .collect();

                elements.extend(layer_surfaces);

                if should_blur {
                    if let Some(engine_lock) = blur_engine {
                        let corner_radius = if let Ok(engine) = engine_lock.lock() {
                            engine.params.corner_radius
                        } else {
                            0.0
                        };
                        let blur_area = Rectangle::new(geo.loc, geo.size);

                        layer
                            .user_data()
                            .insert_if_missing_threadsafe(|| LayerBlurId(Id::new()));
                        let blur_id = layer.user_data().get::<LayerBlurId>().unwrap().0.clone();

                        let blur_elem = BlurRenderElement::new(
                            blur_id,
                            blur_area,
                            corner_radius,
                            engine_lock.clone(),
                        );
                        elements.push(CustomRenderElements::Blur(blur_elem));
                    }
                }
            }
        }
    }
    elements
}

