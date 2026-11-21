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

use crate::compositor::window_animation_frame;
use crate::rendering::animation::{render_animated_window, WindowAnimationRenderParams};
use crate::rendering::blur::{BlurEngine, BlurRenderElement};
use crate::rendering::titlebar;
use common::TITLEBAR_HEIGHT;

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
/// Automatically delegates to [`render_animated_window`] whenever a window transition is active.
pub fn render_window(
    renderer: &mut GlowRenderer,
    window: &Window,
    location: Point<i32, Logical>,
    scale: Scale<f64>,
    blur_engine: Option<&Arc<Mutex<BlurEngine>>>,
    pointer_location: Point<f64, Logical>,
) -> Vec<CustomRenderElements> {
    let (should_blur, opacity, configured_corner_radius) = if let Some(engine_lock) = blur_engine {
        if let Ok(mut engine) = engine_lock.lock() {
            let matched = engine.matches_window(window);
            (
                matched,
                if matched {
                    engine.params.window_opacity
                } else {
                    1.0
                },
                engine.params.corner_radius,
            )
        } else {
            (false, 1.0, common::CORNER_RADIUS_F32)
        }
    } else {
        (false, 1.0, common::CORNER_RADIUS_F32)
    };
    let corner_radius = corner_radius_for_window(window, configured_corner_radius);

    // Delegate to the animation renderer while any transition is in flight
    // (open, close, minimize, restore or fullscreen/maximize geometry change).
    let frame = window_animation_frame(window);
    if frame.active {
        return render_animated_window(
            renderer,
            window,
            location,
            scale,
            blur_engine,
            WindowAnimationRenderParams {
                scale_factor: frame.scale_factor,
                alpha_factor: frame.alpha_factor,
                target_rect: frame.target_rect,
                base_opacity: opacity,
                should_blur,
                corner_radius,
            },
        );
    }

    // Fast path for static/unanimated windows
    let mut elements = Vec::new();

    // The frame is drawn by the compositor, never by GTK/client-side decoration.
    if let Some(frame) = titlebar::render(
        renderer,
        window,
        location,
        scale,
        pointer_location,
        should_blur,
        opacity,
    ) {
        elements.push(frame);
    }
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

    // 2. Blur backdrop element (placed behind window surfaces and server-side titlebar)
    if should_blur {
        if let Some(engine_lock) = blur_engine {
            let is_blur_enabled = engine_lock
                .lock()
                .map(|e| e.params.enabled)
                .unwrap_or(false);

            if is_blur_enabled {
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

                let has_floating_server_titlebar = window
                    .toplevel()
                    .map(|toplevel| {
                        let state = toplevel.current_state();
                        state.decoration_mode
                            == Some(
                                smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode::ServerSide,
                            ) && !state.states.contains(xdg_toplevel::State::Maximized)
                            && !state.states.contains(xdg_toplevel::State::Fullscreen)
                    })
                    .unwrap_or(false);

                // For floating windows with server-side decorations, the blur area extends
                // upward to include the titlebar strip so it shares the frosted glass backdrop.
                let blur_area = if has_floating_server_titlebar {
                    Rectangle::new(
                        blur_loc - Point::from((0, TITLEBAR_HEIGHT)),
                        blur_size + smithay::utils::Size::from((0, TITLEBAR_HEIGHT)),
                    )
                } else {
                    Rectangle::new(blur_loc, blur_size)
                };

                window
                    .user_data()
                    .insert_if_missing(|| WindowBlurId(Id::new()));
                let blur_id = window.user_data().get::<WindowBlurId>().unwrap().0.clone();

                let blur_elem =
                    BlurRenderElement::new(blur_id, blur_area, corner_radius, engine_lock.clone());
                elements.push(CustomRenderElements::Blur(blur_elem));
            }
        }
    }

    elements
}
