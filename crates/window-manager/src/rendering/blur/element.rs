use smithay::{
    backend::renderer::{
        element::{Element, Id, Kind, RenderElement, UnderlyingStorage},
        gles::GlesError,
        glow::{GlowFrame, GlowRenderer},
        utils::{CommitCounter, DamageSet, OpaqueRegions},
    },
    utils::{Buffer, Logical, Physical, Point, Rectangle, Scale, Transform},
};

use std::borrow::Borrow;
use std::sync::{Arc, Mutex};

use super::engine::BlurEngine;
use smithay::backend::renderer::gles::GlesFrame;

/// A render element that applies background Dual Kawase frosted glass blur
/// behind transparent/translucent windows and layer shell surfaces
#[derive(Debug, Clone)]
pub struct BlurRenderElement {
    id: Id,
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
        // The blur does not have a commit of its own, the element is recreated for
        // every render frame anyway.
        CommitCounter::default()
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
        // The blur is not a surface of its own, it renders whatever is drawn
        // below it. Reporting damage on every frame would force the whole output
        // to be redrawn - and re-blurred - 60 times per second, which makes the
        // compositor (and with it the whole system) crawl.
        //
        // The element therefore only reports damage when the region has to be
        // rendered from scratch anyway, i.e. when the element is drawn for the
        // first time or when its background cache is gone. In that case the whole
        // region is damaged, so that the background below the element is drawn
        // again and can be captured without any leftovers.
        //
        // Every other frame the element is drawn with the partial damage of the
        // region, so only the parts that actually changed are re-blurred.
        let geometry = self.geometry(scale);

        let cached = self
            .blur_engine
            .lock()
            .map(|engine| engine.has_backdrop_cache(&self.id, geometry))
            .unwrap_or(false);

        if commit.is_none() || !cached {
            DamageSet::from_slice(&[Rectangle::from_size(geometry.size)])
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
        damage: &[Rectangle<i32, Physical>],
        _opaque_regions: &[Rectangle<i32, Physical>],
    ) -> Result<(), GlesError> {
        let projection: [f32; 9] = {
            let gles: &GlesFrame<'_, '_> = (*frame).borrow();
            *gles.projection()
        };

        // The damage is relative to the element, the blur works with absolute
        // framebuffer coordinates.
        let absolute_damage: Vec<Rectangle<i32, Physical>> = damage
            .iter()
            .map(|rect| {
                Rectangle::new(
                    (dst.loc.x + rect.loc.x, dst.loc.y + rect.loc.y).into(),
                    rect.size,
                )
            })
            .collect();

        frame.with_context(|glow| {
            if let Ok(mut engine) = self.blur_engine.lock() {
                engine.render_blur(
                    glow,
                    &self.id,
                    dst,
                    &absolute_damage,
                    self.corner_radius,
                    projection,
                );
            }
        })?;
        Ok(())
    }

    fn underlying_storage(&self, _renderer: &mut GlowRenderer) -> Option<UnderlyingStorage<'_>> {
        None
    }
}
