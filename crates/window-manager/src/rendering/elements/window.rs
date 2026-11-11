use std::sync::{Arc, Mutex};

use smithay::{
    backend::renderer::{
        element::{surface::WaylandSurfaceRenderElement, AsRenderElements, Id},
        glow::GlowRenderer,
    },
    desktop::Window,
    utils::{Logical, Point, Rectangle, Scale},
};

use crate::rendering::blur::{BlurEngine, BlurRenderElement};

use super::types::{CustomRenderElements, WindowBlurId};

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
                (
                    true,
                    engine.params.window_opacity,
                    engine.params.corner_radius,
                )
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
            let bbox = window.bbox();
            let geo = window.geometry();
            let blur_size = if geo.size.w > 0 && geo.size.h > 0 {
                geo.size
            } else {
                bbox.size
            };
            let blur_loc = if geo.size.w > 0 && geo.size.h > 0 {
                location
            } else {
                location + bbox.loc
            };
            let blur_area = Rectangle::new(blur_loc, blur_size);

            window
                .user_data()
                .insert_if_missing(|| WindowBlurId(Id::new()));
            let blur_id = window.user_data().get::<WindowBlurId>().unwrap().0.clone();

            let blur_elem =
                BlurRenderElement::new(blur_id, blur_area, corner_radius, engine_lock.clone());
            elements.push(CustomRenderElements::Blur(blur_elem));
        }
    }

    elements
}
