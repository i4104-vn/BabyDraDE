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

use crate::rendering::blur::{BlurEngine, BlurRenderElement};
use crate::rendering::elements::types::{ConstrainedSurface, CustomRenderElements, WindowBlurId};

/// Parameters specifying animation progress and blur properties for a transition frame.
#[derive(Clone, Copy, Debug)]
pub struct WindowAnimationRenderParams {
    pub scale_factor: f64,
    pub alpha_factor: f32,
    pub base_opacity: f32,
    pub should_blur: bool,
    pub corner_radius: f32,
}

/// Renders a window during an active animation transition (opening or closing).
///
/// Anchors scaling strictly at the window's geometric center so that closing windows
/// shrink inwards and opening windows expand outward.
pub fn render_animated_window(
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
    let base_size = if bbox.size.w > 0 && bbox.size.h > 0 {
        bbox.size
    } else if geo.size.w > 0 && geo.size.h > 0 {
        geo.size
    } else {
        return elements;
    };

    // Calculate window center in logical coordinates
    let bbox_origin_x = location.x as f64 + bbox.loc.x as f64;
    let bbox_origin_y = location.y as f64 + bbox.loc.y as f64;
    let center_x = bbox_origin_x + (base_size.w as f64) / 2.0;
    let center_y = bbox_origin_y + (base_size.h as f64) / 2.0;

    // Shrink/expand symmetrically from the exact center
    let new_w = ((base_size.w as f64) * params.scale_factor).max(4.0);
    let new_h = ((base_size.h as f64) * params.scale_factor).max(4.0);
    let new_x = center_x - new_w / 2.0;
    let new_y = center_y - new_h / 2.0;

    let constrain = Rectangle::<i32, Logical>::new(
        Point::from((new_x.round() as i32, new_y.round() as i32)),
        Size::from(((new_w.round() as i32).max(4), (new_h.round() as i32).max(4))),
    );

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
                params.corner_radius * (params.scale_factor as f32),
                engine_lock.clone(),
            );
            elements.push(CustomRenderElements::Blur(blur_elem));
        }
    }

    elements
}
