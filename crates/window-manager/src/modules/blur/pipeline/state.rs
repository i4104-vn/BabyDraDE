//! Blur pipeline state, setup and frame rendering.

use glow::HasContext;
use smithay::utils::{Physical, Rectangle};

use common::constants::MAX_BLUR_PASSES;

use super::super::config::BlurParams;
use super::super::models::GlStateSnapshot;
use super::super::shaders::{
    compile_composite, compile_downsample, compile_upsample, setup_tex_params, CompositeProgram,
    DownsampleProgram, UpsampleProgram, QUAD_VERTICES,
};
use super::projection::project_rect_to_gl;
use super::{run_blur, run_composite, BlurCache, BlurPassTarget};

impl BlurPipeline {
    /// Creates a new Dual Kawase BlurPipeline
    ///
    /// # Safety
    ///
    /// Must be invoked while the OpenGL/EGL context is current.
    // SAFETY: GL context is current and object names come from this pipeline.
    pub(crate) unsafe fn new(gl: &glow::Context) -> Result<Self, String> {
        let downsample_prog = compile_downsample(gl)?;
        let upsample_prog = compile_upsample(gl)?;
        let composite_prog = compile_composite(gl)?;

        let vbo = gl.create_buffer()?;
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        let bytes: &[u8] = core::slice::from_raw_parts(
            QUAD_VERTICES.as_ptr() as *const u8,
            QUAD_VERTICES.len() * core::mem::size_of::<f32>(),
        );
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes, glow::STATIC_DRAW);
        gl.bind_buffer(glow::ARRAY_BUFFER, None);

        let mut pass_targets = Vec::with_capacity(MAX_BLUR_PASSES);
        for _ in 0..MAX_BLUR_PASSES {
            pass_targets.push(BlurPassTarget {
                texture: gl.create_texture()?,
                framebuffer: gl.create_framebuffer()?,
            });
        }

        Ok(Self {
            downsample_prog,
            upsample_prog,
            composite_prog,
            vbo,
            pass_targets,
            allocated_w: 0,
            allocated_h: 0,
        })
    }

    // SAFETY: GL context is current and object names come from this pipeline.
    pub(super) unsafe fn ensure_texture_sizes(
        &mut self,
        gl: &glow::Context,
        w: i32,
        h: i32,
    ) -> bool {
        if self.allocated_w >= w && self.allocated_h >= h {
            return true;
        }

        let previous_framebuffer =
            std::num::NonZeroU32::new(gl.get_parameter_i32(glow::FRAMEBUFFER_BINDING) as u32)
                .map(glow::NativeFramebuffer);

        let new_w = w.max(self.allocated_w).max(256);
        let new_h = h.max(self.allocated_h).max(256);

        let mut framebuffers_complete = true;
        for (index, target) in self.pass_targets.iter().enumerate() {
            let divisor = 1_i32 << (index + 1);
            let width = (new_w / divisor).max(1);
            let height = (new_h / divisor).max(1);
            gl.bind_texture(glow::TEXTURE_2D, Some(target.texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width,
                height,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
            setup_tex_params(gl);

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(target.framebuffer));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(target.texture),
                0,
            );
            framebuffers_complete &=
                gl.check_framebuffer_status(glow::FRAMEBUFFER) == glow::FRAMEBUFFER_COMPLETE;
        }

        gl.bind_texture(glow::TEXTURE_2D, None);
        gl.bind_framebuffer(glow::FRAMEBUFFER, previous_framebuffer);

        if framebuffers_complete {
            self.allocated_w = new_w;
            self.allocated_h = new_h;
        }

        framebuffers_complete
    }
}

pub(crate) struct BlurPipeline {
    pub(super) downsample_prog: DownsampleProgram,
    pub(super) upsample_prog: UpsampleProgram,
    pub(super) composite_prog: CompositeProgram,
    pub(super) vbo: glow::Buffer,

    pub(super) pass_targets: Vec<BlurPassTarget>,

    pub(super) allocated_w: i32,
    pub(super) allocated_h: i32,
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
        run_blur(gl, self, region_w, region_h, params, cache);

        // 3. Composite the blurred result over the full region.
        run_composite(
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
