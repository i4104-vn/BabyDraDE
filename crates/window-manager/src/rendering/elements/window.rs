use std::sync::{Arc, Mutex};

use smithay::{
    backend::renderer::{
        element::{surface::WaylandSurfaceRenderElement, AsRenderElements, Id},
        glow::GlowRenderer,
    },
    desktop::Window,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel,
    utils::{Logical, Point, Rectangle, Scale},
};

use crate::compositor::window_animation_factors;
use crate::rendering::animation::{render_animated_window, WindowAnimationRenderParams};
use crate::rendering::blur::{BlurEngine, BlurRenderElement};

use super::types::{CustomRenderElements, WindowBlurId};

/// Full-bleed windows must retain the application's own edge treatment.
fn corner_radius_for_window(window: &Window, floating_radius: f32) -> f32 {
    let Some(toplevel) = window.toplevel() else {
        return floating_radius;
    };

    let states = toplevel.current_state().states;
    if states.contains(xdg_toplevel::State::Maximized)
        || states.contains(xdg_toplevel::State::Fullscreen)
    {
        0.0
    } else {
        floating_radius
    }
}

/// Renders native Wayland client surface elements (including CSD/GTK titlebar & decorations)
/// and attaches a Dual Kawase blur backdrop behind windows configured for blur.
///
/// Automatically delegates to [`render_animated_window`] when an opening or closing transition is active.
pub fn render_window(
    renderer: &mut GlowRenderer,
    window: &Window,
    location: Point<i32, Logical>,
    scale: Scale<f64>,
    blur_engine: Option<&Arc<Mutex<BlurEngine>>>,
) -> Vec<CustomRenderElements> {
    let (should_blur, opacity, configured_corner_radius) = if let Some(engine_lock) = blur_engine {
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
    let corner_radius = corner_radius_for_window(window, configured_corner_radius);

    // If window is animating (open zoom-in or close shrink-to-center + fade), delegate to animation renderer
    let (scale_factor, alpha_factor) = window_animation_factors(window);
    if scale_factor < 0.999 || alpha_factor < 0.999 {
        return render_animated_window(
            renderer,
            window,
            location,
            scale,
            blur_engine,
            WindowAnimationRenderParams {
                scale_factor,
                alpha_factor,
                base_opacity: opacity,
                should_blur,
                corner_radius,
            },
        );
    }

    // Fast path for static/unanimated windows
    let mut elements = Vec::new();
    let render_loc = location - window.geometry().loc;

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
