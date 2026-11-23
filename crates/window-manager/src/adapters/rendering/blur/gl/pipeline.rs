//! Dual Kawase blur pipeline: capture, blur and composite a backdrop region.

mod blur;
mod cache;
mod composite;
mod downsample;
mod setup;
mod upsample;

pub(crate) use cache::BlurCache;

use glow::HasContext;
use smithay::utils::{Physical, Rectangle};

use super::gl_state::GlStateSnapshot;
use super::shaders::{CompositeProgram, DownsampleProgram, UpsampleProgram};
use crate::adapters::rendering::blur::config::BlurParams;

pub(crate) use super::projection::project_rect_to_gl;

use setup::BlurPassTarget;

pub(crate) struct BlurPipeline {
    downsample_prog: DownsampleProgram,
    upsample_prog: UpsampleProgram,
    composite_prog: CompositeProgram,
    vbo: glow::Buffer,

    pass_targets: Vec<BlurPassTarget>,

    allocated_w: i32,
    allocated_h: i32,
}

impl BlurPipeline {
    /// Renders the Dual Kawase blur backdrop over `dst`.
    ///
    /// Every call captures the current framebuffer content, blurs it, and
    /// composites the result. This keeps the blur correct even when only a
    /// small part of the region changes on a given frame.
    ///
    /// # Safety
    ///
    /// Must be invoked while the OpenGL/EGL context is current.
    #[allow(clippy::too_many_arguments)]
    // SAFETY: GL context is current and object names come from this pipeline.
    pub(crate) unsafe fn render_blur(
        &mut self,
        gl: &glow::Context,
        dst: Rectangle<i32, Physical>,
        corner_radius: f32,
        projection: [f32; 9],
        params: &BlurParams,
        cache: &mut BlurCache,
    ) {
        if dst.size.w <= 8 || dst.size.h <= 8 || !params.enabled {
            cache.region = dst;
            return;
        }

        let gl_state = GlStateSnapshot::capture(
            gl,
            [
                self.downsample_prog.a_position,
                self.upsample_prog.a_position,
                self.composite_prog.a_position,
            ],
        );
        let saved_fbo = gl_state.framebuffer;
        let saved_viewport = gl_state.viewport;

        let screen_w = saved_viewport[2];
        let screen_h = saved_viewport[3];

        if screen_w <= 0 || screen_h <= 0 {
            cache.region = dst;
            return;
        }

        // Offscreen passes must not inherit the compositor's output damage scissor.
        gl.disable(glow::SCISSOR_TEST);

        // The region in physical framebuffer coordinates.
        let (raw_x, raw_y, raw_w, raw_h) = project_rect_to_gl(dst, &projection, screen_w, screen_h);

        let x0 = raw_x.max(0);
        let y0 = raw_y.max(0);
        let x1 = (raw_x + raw_w).min(screen_w);
        let y1 = (raw_y + raw_h).min(screen_h);
        let region_w = x1 - x0;
        let region_h = y1 - y0;

        if region_w <= 8 || region_h <= 8 {
            cache.region = dst;
            return;
        }

        // Resize per-element textures when the region changes size.
        if cache.region != dst || cache.size != (region_w, region_h) {
            if !cache.resize(gl, region_w, region_h) {
                cache.region = dst;
                return;
            }
            cache.region = dst;
        }

        // Shared pass textures must be large enough for this region.
        if !self.ensure_texture_sizes(gl, region_w, region_h) {
            return;
        }

        // 1. Capture the full background behind the element.
        //    Always read a fresh copy so there is no feedback loop where the
        //    blurred result gets blurred again on subsequent frames.
        gl.bind_framebuffer(glow::FRAMEBUFFER, saved_fbo);
        gl.bind_texture(glow::TEXTURE_2D, Some(cache.backdrop));
        gl.copy_tex_sub_image_2d(glow::TEXTURE_2D, 0, 0, 0, x0, y0, region_w, region_h);
        gl.bind_texture(glow::TEXTURE_2D, None);

        // 2. Blur the captured backdrop into cache.blurred.
        blur::run(gl, self, region_w, region_h, params, cache);

        // 3. Composite the blurred result over the full region.
        composite::run(
            gl,
            self,
            saved_fbo,
            (x0, y0, region_w, region_h),
            (x0, y0, region_w, region_h),
            corner_radius,
            params,
            cache,
        );
    }
}
