//! Dual Kawase blur pipeline: capture, blur and composite a backdrop region.

use glow::HasContext;
use smithay::utils::{Physical, Rectangle};

use common::constants::MAX_BLUR_PASSES;

use super::config::BlurParams;
use super::gl_state::GlStateSnapshot;
use super::shaders::{
    compile_composite, compile_downsample, compile_upsample, setup_tex_params, QUAD_VERTICES,
    CompositeProgram, DownsampleProgram, UpsampleProgram,
};

pub(crate) mod passes;

pub(super) use crate::modules::blur::models::BlurCache;
#[allow(unused_imports)]
pub(super) use passes::{run_blur, run_composite, run_downsample, run_upsample, BlurPassTarget, GlRect};

/// Projects a destination rectangle from Wayland oriented coordinates to OpenGL physical
/// framebuffer coordinates using Smithay's exact projection matrix for the current frame.
pub(crate) fn project_rect_to_gl(
    dst: Rectangle<i32, Physical>,
    projection: &[f32; 9],
    screen_w: i32,
    screen_h: i32,
) -> (i32, i32, i32, i32) {
    let x0 = dst.loc.x as f32;
    let y0 = dst.loc.y as f32;
    let x1 = x0 + dst.size.w as f32;
    let y1 = y0 + dst.size.h as f32;

    let corners = [(x0, y0), (x1, y0), (x0, y1), (x1, y1)];

    let mut min_gx = f32::INFINITY;
    let mut max_gx = f32::NEG_INFINITY;
    let mut min_gy = f32::INFINITY;
    let mut max_gy = f32::NEG_INFINITY;

    for (x, y) in corners {
        // Column-major Matrix3 * vec3(x, y, 1.0)
        let mut rx = projection[0] * x + projection[3] * y + projection[6];
        let mut ry = projection[1] * x + projection[4] * y + projection[7];
        let rw = projection[2] * x + projection[5] * y + projection[8];

        if rw.abs() > 1e-6 {
            rx /= rw;
            ry /= rw;
        }

        // OpenGL NDC [-1.0, 1.0] -> Viewport [0, screen_size]
        let gx = (rx + 1.0) * 0.5 * screen_w as f32;
        let gy = (ry + 1.0) * 0.5 * screen_h as f32;

        min_gx = min_gx.min(gx);
        max_gx = max_gx.max(gx);
        min_gy = min_gy.min(gy);
        max_gy = max_gy.max(gy);
    }

    let gl_x = min_gx.round() as i32;
    let gl_y = min_gy.round() as i32;
    let gl_w = (max_gx - min_gx).round() as i32;
    let gl_h = (max_gy - min_gy).round() as i32;

    (gl_x, gl_y, gl_w, gl_h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_rect_to_gl_normal() {
        // Identity projection for 1920x1080 display
        // M00=2/1920, M11=2/1080, M02=-1, M12=-1 (Y up)
        let mut proj = [0.0f32; 9];
        proj[0] = 2.0 / 1920.0;
        proj[4] = 2.0 / 1080.0;
        proj[6] = -1.0;
        proj[7] = -1.0;
        proj[8] = 1.0;

        let dst = Rectangle::new((100, 200).into(), (400, 300).into());
        let (x, y, w, h) = project_rect_to_gl(dst, &proj, 1920, 1080);
        assert_eq!((x, y, w, h), (100, 200, 400, 300));
    }

    #[test]
    fn test_project_rect_to_gl_rotated_90() {
        // Rotated projection for 1600x2560 hardware panel displayed at 2560x1600 (Transform::_270 render transform)
        // With gx = 1600 - ly, gy = lx
        let mut proj = [0.0f32; 9];
        // ndc_x = 1.0 - 2.0 * y / 1600.0 => proj[3] = -2.0/1600.0, proj[6] = 1.0 (row 0)
        proj[3] = -2.0 / 1600.0;
        proj[6] = 1.0;
        // ndc_y = 2.0 * x / 2560.0 - 1.0 => proj[1] = 2.0/2560.0, proj[7] = -1.0 (row 1)
        proj[1] = 2.0 / 2560.0;
        proj[7] = -1.0;
        proj[8] = 1.0;

        // Window at top-left: x=0, y=0, w=400, h=300
        let dst = Rectangle::new((0, 0).into(), (400, 300).into());
        let (x, y, w, h) = project_rect_to_gl(dst, &proj, 1600, 2560);
        // gx range = [1600 - 300, 1600] = [1300, 1600]
        // gy range = [0, 400]
        assert_eq!((x, y, w, h), (1300, 0, 300, 400));
    }
}
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
