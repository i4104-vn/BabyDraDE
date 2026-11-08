use std::borrow::BorrowMut;

use smithay::{
    backend::renderer::{
        element::{
            render_elements, surface::WaylandSurfaceRenderElement, Element, Id, Kind, RenderElement,
        },
        gles::{element::PixelShaderElement, GlesError, GlesFrame, Uniform},
        glow::{GlowFrame, GlowRenderer},
        utils::{CommitCounter, DamageSet, OpaqueRegions},
        Renderer,
    },
    utils::{Buffer, Physical, Point, Rectangle, Scale, Transform},
};

use super::corners::CornerShader;

pub struct WindowRenderElement<R: Renderer> {
    inner: WaylandSurfaceRenderElement<R>,
    radius: f32,
}

impl<R: Renderer> WindowRenderElement<R> {
    pub fn new(inner: WaylandSurfaceRenderElement<R>, radius: f32) -> Self {
        Self { inner, radius }
    }
}

impl Element for WindowRenderElement<GlowRenderer> {
    fn id(&self) -> &Id {
        self.inner.id()
    }

    fn current_commit(&self) -> CommitCounter {
        self.inner.current_commit()
    }

    fn src(&self) -> Rectangle<f64, Buffer> {
        self.inner.src()
    }

    fn geometry(&self, scale: Scale<f64>) -> Rectangle<i32, Physical> {
        self.inner.geometry(scale)
    }

    fn location(&self, scale: Scale<f64>) -> Point<i32, Physical> {
        self.inner.location(scale)
    }

    fn transform(&self) -> Transform {
        self.inner.transform()
    }

    fn damage_since(
        &self,
        scale: Scale<f64>,
        commit: Option<CommitCounter>,
    ) -> DamageSet<i32, Physical> {
        self.inner.damage_since(scale, commit)
    }

    fn opaque_regions(&self, _scale: Scale<f64>) -> OpaqueRegions<i32, Physical> {
        OpaqueRegions::default()
    }

    fn alpha(&self) -> f32 {
        self.inner.alpha()
    }

    fn kind(&self) -> Kind {
        self.inner.kind()
    }
}

impl RenderElement<GlowRenderer> for WindowRenderElement<GlowRenderer> {
    fn draw(
        &self,
        frame: &mut GlowFrame<'_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        opaque_regions: &[Rectangle<i32, Physical>],
    ) -> Result<(), GlesError> {
        if self.radius > 0.0 {
            let size = self
                .geometry(Scale::from(1.0))
                .size
                .to_logical(Scale::from(1));
            let framegl = <GlowFrame<'_, '_> as BorrowMut<GlesFrame>>::borrow_mut(frame);
            framegl.override_default_tex_program(
                CornerShader::get(framegl.egl_context()),
                vec![
                    Uniform::new("size", [size.w as f32, size.h as f32]),
                    Uniform::new("radius", self.radius),
                ],
            );
            self.inner
                .draw(frame, src, dst, damage, opaque_regions)?;
            <GlowFrame<'_, '_> as BorrowMut<GlesFrame>>::borrow_mut(frame)
                .clear_tex_program_override();
        } else {
            self.inner
                .draw(frame, src, dst, damage, opaque_regions)?;
        }
        Ok(())
    }
}

render_elements! {
    pub CustomRenderElements<=GlowRenderer>;
    Surface=WaylandSurfaceRenderElement<GlowRenderer>,
    Shader=PixelShaderElement,
    Window=WindowRenderElement<GlowRenderer>,
}
