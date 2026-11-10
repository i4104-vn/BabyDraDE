use std::sync::{Arc, Mutex};
use smithay::{
    backend::renderer::{
        element::{Element, Id, Kind, RenderElement, UnderlyingStorage},
        gles::GlesError,
        glow::{GlowFrame, GlowRenderer},
        utils::{CommitCounter, DamageSet, OpaqueRegions},
        Frame,
    },
    utils::{Buffer, Logical, Physical, Point, Rectangle, Scale, Transform},
};

use std::borrow::Borrow;
use smithay::backend::renderer::gles::GlesFrame;
use super::engine::BlurEngine;

/// A render element that applies background Dual Kawase frosted glass blur
/// behind transparent/translucent windows and layer shell surfaces
#[derive(Debug, Clone)]
pub struct BlurRenderElement {
    id: Id,
    commit_counter: CommitCounter,
    area: Rectangle<i32, Logical>,
    corner_radius: f32,
    blur_engine: Arc<Mutex<BlurEngine>>,
}

impl BlurRenderElement {
    pub fn new(
        id: Id,
        area: Rectangle<i32, Logical>,
        corner_radius: f32,
        blur_engine: Arc<Mutex<BlurEngine>>,
    ) -> Self {
        Self {
            id,
            commit_counter: CommitCounter::default(),
            area,
            corner_radius,
            blur_engine,
        }
    }
}

impl Element for BlurRenderElement {
    fn id(&self) -> &Id {
        &self.id
    }

    fn current_commit(&self) -> CommitCounter {
        self.commit_counter
    }

    fn src(&self) -> Rectangle<f64, Buffer> {
        Rectangle::from_size(self.area.size.to_f64().to_buffer(1.0, Transform::Normal))
    }

    fn geometry(&self, scale: Scale<f64>) -> Rectangle<i32, Physical> {
        self.area.to_physical_precise_round(scale)
    }

    fn location(&self, scale: Scale<f64>) -> Point<i32, Physical> {
        self.area.loc.to_physical_precise_round(scale)
    }

    fn damage_since(
        &self,
        scale: Scale<f64>,
        commit: Option<CommitCounter>,
    ) -> DamageSet<i32, Physical> {
        if commit != Some(self.current_commit()) {
            DamageSet::from_slice(&[Rectangle::from_size(self.geometry(scale).size)])
        } else {
            DamageSet::default()
        }
    }

    fn opaque_regions(&self, _scale: Scale<f64>) -> OpaqueRegions<i32, Physical> {
        // Blur backdrops are translucent, never opaque
        OpaqueRegions::default()
    }

    fn alpha(&self) -> f32 {
        1.0
    }

    fn kind(&self) -> Kind {
        Kind::Unspecified
    }
}

impl RenderElement<GlowRenderer> for BlurRenderElement {
    fn draw(
        &self,
        frame: &mut GlowFrame<'_, '_>,
        _src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        _damage: &[Rectangle<i32, Physical>],
        _opaque_regions: &[Rectangle<i32, Physical>],
    ) -> Result<(), GlesError> {
        let transform = frame.transformation();
        let projection: [f32; 9] = {
            let gles: &GlesFrame<'_, '_> = (*frame).borrow();
            *gles.projection()
        };
        frame.with_context(|glow| {
            if let Ok(mut engine) = self.blur_engine.lock() {
                engine.render_blur(glow, dst, self.corner_radius, transform, projection);
            }
        })?;
        Ok(())
    }

    fn underlying_storage(&self, _renderer: &mut GlowRenderer) -> Option<UnderlyingStorage<'_>> {
        None
    }
}
