//! Live window preview shown beside the highlighted dock entry.

use smithay::{
    backend::renderer::{
        element::utils::{ConstrainAlign, ConstrainScaleBehavior, CropRenderElement},
        glow::GlowRenderer,
    },
    desktop::{
        space::{constrain_space_element, ConstrainBehavior, ConstrainReference},
        Window,
    },
    utils::{Logical, Physical, Point, Rectangle, Scale, Size},
};

use crate::modules::shell::switcher::{render::SwitcherLayout, AltTabSwitcher};
use crate::modules::render::elements::ConstrainedSurface;
use crate::modules::render::CustomRenderElements;

use super::super::draw::Rect;

/// Builds the cropped preview element for the currently highlighted entry.
pub(super) fn elements(
    renderer: &mut GlowRenderer,
    switcher: &AltTabSwitcher,
    layout: &SwitcherLayout,
    slide_x: f64,
    scale: Scale<f64>,
) -> Vec<CustomRenderElements> {
    let mut elements = Vec::new();
    // One small live preview beside the active icon; the big centre
    // stage no longer exists.
    if let Some(entry) = switcher.entries.get(layout.selected) {
        let mut preview_rect = layout.preview;
        preview_rect.x += slide_x;
        let crop = Rectangle::<i32, Physical>::new(
            Point::<i32, Physical>::from((
                (preview_rect.x * scale.x).floor() as i32,
                (preview_rect.y * scale.y).floor() as i32,
            )),
            Size::<i32, Physical>::from((
                ((preview_rect.x + preview_rect.width) * scale.x).ceil() as i32
                    - (preview_rect.x * scale.x).floor() as i32,
                ((preview_rect.y + preview_rect.height) * scale.y).ceil() as i32
                    - (preview_rect.y * scale.y).floor() as i32,
            )),
        );

        elements.extend(
            render_window_preview(
                renderer,
                &entry.window,
                preview_rect,
                scale,
                layout.preview_alpha as f32,
                ConstrainScaleBehavior::Zoom,
                ConstrainReference::Geometry,
            )
            .into_iter()
            .filter_map(|preview| {
                CropRenderElement::from_element(preview, scale, crop)
                    .map(CustomRenderElements::DockClipped)
            }),
        );
    }
    elements
}

/// Render a window scaled into `target_rect` (logical coordinates).
///
/// `scale_behavior` controls whether the window is stretched to fill the slot
/// (`Stretch`, used for the center stage) or zoomed to cover it (`Zoom`, used
/// for dock thumbnails so the dark card behind never shows at their edges).
///
/// Uses smithay's `constrain_space_element` pipeline so that the compositor's damage
/// tracker sees the correct physical geometry of every sub-surface element.
fn render_window_preview(
    renderer: &mut GlowRenderer,
    window: &Window,
    target_rect: Rect,
    output_scale: Scale<f64>,
    alpha: f32,
    scale_behavior: ConstrainScaleBehavior,
    reference: ConstrainReference,
) -> Vec<ConstrainedSurface> {
    // Guard: window must have a valid bounding-box so constrain can compute scale.
    let bbox = window.bbox();
    if bbox.size.w <= 0 || bbox.size.h <= 0 {
        // Try geometry as fallback.
        let geo = window.geometry();
        if geo.size.w <= 0 || geo.size.h <= 0 {
            return Vec::new();
        }
    }

    // The constrain rect in logical coordinates (what the layout computed).
    let constrain = Rectangle::<i32, Logical>::new(
        Point::<i32, Logical>::from((target_rect.x.round() as i32, target_rect.y.round() as i32)),
        Size::<i32, Logical>::from((
            (target_rect.width.round() as i32).max(4),
            (target_rect.height.round() as i32).max(4),
        )),
    );

    // `location` anchors the chosen reference rectangle (window geometry, not
    // the bbox with client-side shadows) at the top-left of the slot. Smithay
    // then scales/crops it into `constrain`, so CSD drop shadows outside the
    // geometry never become a dark rim around the preview.
    let location: Point<i32, Logical> = constrain.loc;

    let behavior = ConstrainBehavior {
        reference,
        behavior: scale_behavior,
        align: ConstrainAlign::CENTER,
    };

    constrain_space_element::<GlowRenderer, Window, ConstrainedSurface>(
        renderer,
        window,
        location,
        alpha,
        output_scale,
        constrain,
        behavior,
    )
    .collect()
}
