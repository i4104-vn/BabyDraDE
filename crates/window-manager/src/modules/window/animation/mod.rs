use std::sync::{Arc, Mutex};

use smithay::{
    backend::renderer::{
        element::{
            utils::{ConstrainAlign, ConstrainScaleBehavior},
            Id,
        },
        glow::GlowRenderer,
    },
    desktop::{
        space::{constrain_space_element, ConstrainBehavior, ConstrainReference},
        Window,
    },
    utils::{Logical, Point, Rectangle, Scale, Size},
};

use crate::modules::blur::{BlurEngine, BlurRenderElement};
use crate::modules::render::elements::{ConstrainedSurface, CustomRenderElements, WindowBlurId};

pub(crate) use crate::modules::window::models::animation::{
    GeometryTransition, WindowAnimation, WindowAnimationRenderParams,
};

/// Renders a window during any active animation (opening, closing, minimize,
/// restore or a fullscreen/maximize geometry transition).
///
/// Scale-only transitions anchor at the window's geometric center. Geometry
/// transitions instead render into their interpolated target rectangle.
pub(crate) fn render_animated_window(
    renderer: &mut GlowRenderer,
    window: &Window,
    location: Point<i32, Logical>,
    scale: Scale<f64>,
    blur_engine: Option<&Arc<Mutex<BlurEngine>>>,
    params: WindowAnimationRenderParams,
) -> Vec<CustomRenderElements> {
    let mut elements = Vec::new();

    let bbox = window.bbox();
    let geo = window.geometry();
    let base_size = match true {
        _ if bbox.size.w > 0 && bbox.size.h > 0 => bbox.size,
        _ if geo.size.w > 0 && geo.size.h > 0 => geo.size,
        _ => return elements,
    };

    // Calculate window center in logical coordinates
    let bbox_origin_x = location.x as f64 + bbox.loc.x as f64;
    let bbox_origin_y = location.y as f64 + bbox.loc.y as f64;
    let center_x = bbox_origin_x + (base_size.w as f64) / 2.0;
    let center_y = bbox_origin_y + (base_size.h as f64) / 2.0;

    // Geometry transitions provide their own interpolated destination rect,
    // including position, so the window moves and resizes in one smooth step.
    // Scale-only transitions (open/close/minimize/restore) keep the historic
    // center-anchored behaviour.
    let (constrain, effective_scale) = match params.target_rect {
        Some(target) => {
            let constrain = Rectangle::<i32, Logical>::new(
                Point::from((target.loc.x.round() as i32, target.loc.y.round() as i32)),
                Size::from((
                    (target.size.w.round() as i32).max(4),
                    (target.size.h.round() as i32).max(4),
                )),
            );
            let scale = if base_size.w > 0 {
                constrain.size.w as f64 / base_size.w as f64
            } else {
                1.0
            };
            (constrain, scale)
        }
        None => {
            let new_w = ((base_size.w as f64) * params.scale_factor).max(4.0);
            let new_h = ((base_size.h as f64) * params.scale_factor).max(4.0);
            let new_x = center_x - new_w / 2.0;
            let new_y = center_y - new_h / 2.0;
            let constrain = Rectangle::<i32, Logical>::new(
                Point::from((new_x.round() as i32, new_y.round() as i32)),
                Size::from(((new_w.round() as i32).max(4), (new_h.round() as i32).max(4))),
            );
            (constrain, params.scale_factor)
        }
    };

    let behavior = ConstrainBehavior {
        reference: ConstrainReference::BoundingBox,
        behavior: ConstrainScaleBehavior::Stretch,
        align: ConstrainAlign::CENTER,
    };

    let effective_opacity = (params.base_opacity * params.alpha_factor).clamp(0.0, 1.0);

    // Render scaled client window surfaces
    let surfaces: Vec<CustomRenderElements> =
        constrain_space_element::<GlowRenderer, Window, ConstrainedSurface>(
            renderer,
            window,
            constrain.loc,
            effective_opacity,
            scale,
            constrain,
            behavior,
        )
        .map(CustomRenderElements::Constrained)
        .collect();

    elements.extend(surfaces);

    // Attach matching blur backdrop scaled with the window
    if params.should_blur {
        if let Some(engine_lock) = blur_engine {
            window
                .user_data()
                .insert_if_missing(|| WindowBlurId(Id::new()));
            let blur_id = window.user_data().get::<WindowBlurId>().unwrap().0.clone();

            let blur_elem = BlurRenderElement::new(
                blur_id,
                constrain,
                params.corner_radius * (effective_scale as f32),
                engine_lock.clone(),
            );
            elements.push(CustomRenderElements::Blur(blur_elem));
        }
    }

    elements
}

pub(crate) mod frame;
pub(crate) mod lifecycle;

pub(crate) use frame::window_animation_frame;
pub(crate) use lifecycle::{
    cancel_minimizing_animation, is_window_exiting, start_window_minimizing, start_window_opening,
    start_window_restoring, start_window_transition,
};

use crate::modules::window::state_data::lock_window_state;
use crate::shared::types::geometry::{
    Point as PurePoint, Rectangle as PureRectangle, Size as PureSize,
};
use crate::shared::types::WindowVisualState;
