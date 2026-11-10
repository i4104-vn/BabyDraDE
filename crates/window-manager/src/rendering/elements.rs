use std::sync::{Arc, Mutex};

use smithay::{
    backend::renderer::{
        element::{
            memory::MemoryRenderBufferRenderElement, render_elements,
            surface::WaylandSurfaceRenderElement, AsRenderElements, Id,
        },
        glow::GlowRenderer,
    },
    desktop::{layer_map_for_output, Window},
    output::Output,
    utils::{Logical, Point, Rectangle, Scale},
    wayland::shell::wlr_layer::Layer,
};

use super::blur::{BlurEngine, BlurRenderElement};

#[derive(Debug, Clone)]
struct WindowBlurId(Id);

#[derive(Debug, Clone)]
struct LayerBlurId(Id);

render_elements! {
    pub CustomRenderElements<=GlowRenderer>;
    Surface=WaylandSurfaceRenderElement<GlowRenderer>,
    Memory=MemoryRenderBufferRenderElement<GlowRenderer>,
    Blur=BlurRenderElement,
}

/// Renders native Wayland client surface elements (including CSD/GTK titlebar & decorations)
/// and attaches a Dual Kawase blur backdrop behind windows configured for blur.
pub fn render_window(
    renderer: &mut GlowRenderer,
    window: &Window,
    location: Point<i32, Logical>,
    scale: Scale<f64>,
    blur_engine: Option<&Arc<Mutex<BlurEngine>>>,
) -> Vec<CustomRenderElements> {
    let mut elements = Vec::new();
    let render_loc = location - window.geometry().loc;

    let (should_blur, opacity, corner_radius) = if let Some(engine_lock) = blur_engine {
        if let Ok(engine) = engine_lock.lock() {
            if engine.matches_window(window) {
                (true, engine.params.window_opacity, engine.params.corner_radius)
            } else {
                (false, 1.0, 0.0)
            }
        } else {
            (false, 1.0, 0.0)
        }
    } else {
        (false, 1.0, 0.0)
    };

    // 1. Wayland window surface elements (in front)
    let surfaces: Vec<CustomRenderElements> = window
        .render_elements::<WaylandSurfaceRenderElement<GlowRenderer>>(
            renderer,
            render_loc.to_physical_precise_round(scale),
            scale,
            opacity,
        )
        .into_iter()
        .map(CustomRenderElements::Surface)
        .collect();

    elements.extend(surfaces);

    // 2. Blur backdrop element (placed behind window surfaces)
    if should_blur {
        if let Some(engine_lock) = blur_engine {
            let geo = window.geometry();
            let blur_area = Rectangle::new(location, geo.size);

            window
                .user_data()
                .insert_if_missing(|| WindowBlurId(Id::new()));
            let blur_id = window
                .user_data()
                .get::<WindowBlurId>()
                .unwrap()
                .0
                .clone();

            let blur_elem = BlurRenderElement::new(
                blur_id,
                blur_area,
                corner_radius,
                engine_lock.clone(),
            );
            elements.push(CustomRenderElements::Blur(blur_elem));
        }
    }

    elements
}

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

                let should_blur = if let Some(engine_lock) = blur_engine {
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
                            if layer.namespace().contains("topbar") {
                                0.0
                            } else {
                                engine.params.corner_radius
                            }
                        } else {
                            0.0
                        };
                        let blur_area = Rectangle::new(geo.loc, geo.size);

                        layer
                            .user_data()
                            .insert_if_missing_threadsafe(|| LayerBlurId(Id::new()));
                        let blur_id = layer
                            .user_data()
                            .get::<LayerBlurId>()
                            .unwrap()
                            .0
                            .clone();

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
