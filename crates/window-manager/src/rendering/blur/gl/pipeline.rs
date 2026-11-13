use glow::HasContext;
use smithay::utils::{Physical, Rectangle};

use super::{
    gl_state::GlStateSnapshot,
    shaders::{
        compile_composite, compile_downsample, compile_upsample, setup_tex_params,
        CompositeProgram, DownsampleProgram, UpsampleProgram, QUAD_VERTICES,
    },
};
use crate::rendering::blur::config::BlurParams;

pub use super::projection::project_rect_to_gl;

const MAX_BLUR_PASSES: usize = 4;

/// Cached state of a single blur element.
///
/// The blur samples the pixels that are drawn *below* the element. Those pixels
/// live in the output framebuffer and cannot be sampled directly, so they are
/// captured into `backdrop` and kept up to date with the damaged parts only.
/// `blurred` holds the result of the dual kawase pass, so that the element can
/// re-composite it without running the whole blur again.
pub struct BlurCache {
    /// The raw background of the blurred region.
    pub(crate) backdrop: glow::Texture,
    /// The blurred result of the region (at half resolution).
    pub(crate) blurred: glow::Texture,
    /// The framebuffer the blurred result is rendered into.
    pub(crate) blurred_fbo: glow::Framebuffer,
    /// Size (in physical pixels) of the region the textures hold.
    pub(crate) size: (i32, i32),
    /// The region the cached background belongs to.
    pub(crate) region: Rectangle<i32, Physical>,
    /// The size for which the driver rejected the offscreen target.
    ///
    /// Remembering it avoids retrying an allocation that cannot succeed (for
    /// example a region larger than `GL_MAX_TEXTURE_SIZE`) on every frame.
    pub(crate) failed_size: Option<(i32, i32)>,
    /// The frame in which the cache was last used.
    pub(crate) last_used: u64,
    /// The amount of video memory the cache occupies.
    pub(crate) bytes: usize,
}

impl BlurCache {
    /// Creates an empty cache.
    ///
    /// # Safety
    ///
    /// Must be invoked while the OpenGL/EGL context is current.
    pub unsafe fn new(gl: &glow::Context) -> Result<Self, String> {
        let backdrop = gl
            .create_texture()
            .map_err(|err| format!("failed to create the blur backdrop texture: {err}"))?;
        let blurred = match gl.create_texture() {
            Ok(texture) => texture,
            Err(err) => {
                gl.delete_texture(backdrop);
                return Err(format!("failed to create the blur result texture: {err}"));
            }
        };
        let blurred_fbo = match gl.create_framebuffer() {
            Ok(framebuffer) => framebuffer,
            Err(err) => {
                gl.delete_texture(backdrop);
                gl.delete_texture(blurred);
                return Err(format!(
                    "failed to create the blur result framebuffer: {err}"
                ));
            }
        };

        Ok(Self {
            backdrop,
            blurred,
            blurred_fbo,
            size: (0, 0),
            region: Rectangle::from_size((0, 0).into()),
            failed_size: None,
            last_used: 0,
            bytes: 0,
        })
    }

    /// Frees the textures of the cache.
    ///
    /// # Safety
    ///
    /// Must be invoked while the OpenGL/EGL context is current.
    pub unsafe fn release(&mut self, gl: &glow::Context) {
        gl.delete_texture(self.backdrop);
        gl.delete_texture(self.blurred);
        gl.delete_framebuffer(self.blurred_fbo);
        self.bytes = 0;
        self.size = (0, 0);
        self.failed_size = None;
    }

    /// (Re-)allocates the textures so that they can hold a region of the given size.
    ///
    /// Returns `false` when the driver did not accept the offscreen target. The
    /// blur has to be skipped in that case, otherwise an uninitialized texture
    /// would be composited over the content of the window.
    ///
    /// # Safety
    ///
    /// Must be invoked while the OpenGL/EGL context is current.
    pub unsafe fn resize(&mut self, gl: &glow::Context, width: i32, height: i32) -> bool {
        let width = width.max(1);
        let height = height.max(1);
        if self.size == (width, height) {
            return true;
        }
        if self.failed_size == Some((width, height)) {
            return false;
        }

        // Textures larger than this are rejected by the driver. Bail out
        // before issuing an allocation that is guaranteed to fail.
        let max_texture_size = gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE).max(0);
        if max_texture_size > 0 && (width > max_texture_size || height > max_texture_size) {
            self.size = (0, 0);
            self.bytes = 0;
            self.failed_size = Some((width, height));
            return false;
        }

        // `resize` is also called from within the render pass, preserve the
        // framebuffer that is currently being rendered to.
        let previous_framebuffer =
            std::num::NonZeroU32::new(gl.get_parameter_i32(glow::FRAMEBUFFER_BINDING) as u32)
                .map(glow::NativeFramebuffer);

        gl.bind_texture(glow::TEXTURE_2D, Some(self.backdrop));
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

        // The blur result is kept at half resolution, the dual kawase pass
        // upsamples back to that resolution anyway.
        let result_width = (width / 2).max(1);
        let result_height = (height / 2).max(1);
        gl.bind_texture(glow::TEXTURE_2D, Some(self.blurred));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            result_width,
            result_height,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(None),
        );
        setup_tex_params(gl);

        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.blurred_fbo));
        gl.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            glow::TEXTURE_2D,
            Some(self.blurred),
            0,
        );
        let complete = gl.check_framebuffer_status(glow::FRAMEBUFFER) == glow::FRAMEBUFFER_COMPLETE;
        gl.bind_framebuffer(glow::FRAMEBUFFER, previous_framebuffer);
        gl.bind_texture(glow::TEXTURE_2D, None);

        // The previous contents are unusable after a failed resize. Resetting
        // the size makes sure a later call cannot mistake a partially
        // allocated texture for a valid cache.
        self.size = (0, 0);
        self.bytes =
            width as usize * height as usize * 4 + result_width as usize * result_height as usize * 4;
        if !complete {
            self.failed_size = Some((width, height));
            return false;
        }

        self.failed_size = None;
        self.size = (width, height);
        true
    }
}

struct BlurPassTarget {
    texture: glow::Texture,
    framebuffer: glow::Framebuffer,
}

/// A rectangle in framebuffer coordinates.
type GlRect = (i32, i32, i32, i32);

pub struct BlurPipeline {
    downsample_prog: DownsampleProgram,
    upsample_prog: UpsampleProgram,
    composite_prog: CompositeProgram,
    vbo: glow::Buffer,

    pass_targets: Vec<BlurPassTarget>,

    allocated_w: i32,
    allocated_h: i32,
}

impl BlurPipeline {
    /// Creates a new Dual Kawase BlurPipeline
    ///
    /// # Safety
    ///
    /// Must be invoked while the OpenGL/EGL context is current.
    pub unsafe fn new(gl: &glow::Context) -> Result<Self, String> {
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

    unsafe fn ensure_texture_sizes(&mut self, gl: &glow::Context, w: i32, h: i32) -> bool {
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
    pub unsafe fn render_blur(
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
        self.blur_backdrop(gl, region_w, region_h, params, cache);

        // 3. Composite the blurred result over the full region.
        self.composite(
            gl,
            saved_fbo,
            (x0, y0, region_w, region_h),
            (x0, y0, region_w, region_h),
            corner_radius,
            params,
            cache,
        );
    }

    /// Runs the dual kawase pass on the cached backdrop and stores the result in
    /// the cache, so that it can be re-composited later on.
    ///
    /// # Safety
    ///
    /// Must be invoked while the OpenGL/EGL context is current.
    unsafe fn blur_backdrop(
        &mut self,
        gl: &glow::Context,
        region_w: i32,
        region_h: i32,
        params: &BlurParams,
        cache: &mut BlurCache,
    ) {
        let passes = params.passes.clamp(1, MAX_BLUR_PASSES);

        // The backdrop cache holds the region at its full extent, the shared pass
        // targets are allocated for the largest region seen so far and only hold
        // the region in a sub rectangle of them.
        let backdrop_scale = (1.0f32, 1.0f32);
        let target_scale = (
            region_w as f32 / self.allocated_w as f32,
            region_h as f32 / self.allocated_h as f32,
        );

        gl.disable(glow::BLEND);
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));

        // 1. Downsample into progressively smaller cached targets.
        gl.use_program(Some(self.downsample_prog.program));
        gl.active_texture(glow::TEXTURE0);
        gl.uniform_1_i32(Some(&self.downsample_prog.u_texture), 0);
        gl.enable_vertex_attrib_array(self.downsample_prog.a_position);
        gl.vertex_attrib_pointer_f32(self.downsample_prog.a_position, 2, glow::FLOAT, false, 8, 0);

        let mut input_w = region_w;
        let mut input_h = region_h;
        let mut input_scale = backdrop_scale;

        for index in 0..passes {
            let divisor = 1_i32 << (index + 1);
            let target_width = (region_w / divisor).max(1);
            let target_height = (region_h / divisor).max(1);

            // With a single pass the result is produced by the downsample step
            // directly, it is written into the per element result texture so that
            // it survives until the next frame.
            let is_final = passes == 1;

            let input_texture = if index == 0 {
                cache.backdrop
            } else {
                self.pass_targets[index - 1].texture
            };
            let target_fbo = if is_final {
                cache.blurred_fbo
            } else {
                self.pass_targets[index].framebuffer
            };

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(target_fbo));
            gl.viewport(0, 0, target_width, target_height);
            gl.bind_texture(glow::TEXTURE_2D, Some(input_texture));
            gl.uniform_2_f32(
                Some(&self.downsample_prog.u_halfpixel),
                0.5 / input_w as f32,
                0.5 / input_h as f32,
            );
            gl.uniform_1_f32(
                Some(&self.downsample_prog.u_offset),
                params.radius * (1_i32 << index) as f32 * 0.5,
            );
            gl.uniform_2_f32(
                Some(&self.downsample_prog.u_scale_uv),
                input_scale.0,
                input_scale.1,
            );
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);

            input_w = (self.allocated_w / (1_i32 << (index + 1))).max(1);
            input_h = (self.allocated_h / (1_i32 << (index + 1))).max(1);
            input_scale = target_scale;
        }
        gl.disable_vertex_attrib_array(self.downsample_prog.a_position);

        // 2. Upsample back to the first target. Each pass reuses the next target.
        gl.use_program(Some(self.upsample_prog.program));
        gl.uniform_1_i32(Some(&self.upsample_prog.u_texture), 0);
        for index in (0..passes.saturating_sub(1)).rev() {
            // The last upsample step writes into the per element result texture.
            let is_final = index == 0;

            let (source, target_fbo) = if is_final {
                (self.pass_targets[index + 1].texture, cache.blurred_fbo)
            } else {
                (
                    self.pass_targets[index + 1].texture,
                    self.pass_targets[index].framebuffer,
                )
            };

            let divisor = 1_i32 << (index + 1);
            let target_width = (region_w / divisor).max(1);
            let target_height = (region_h / divisor).max(1);
            let input_divisor = 1_i32 << (index + 2);
            let input_width = (self.allocated_w / input_divisor).max(1);
            let input_height = (self.allocated_h / input_divisor).max(1);

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(target_fbo));
            gl.viewport(0, 0, target_width, target_height);
            gl.bind_texture(glow::TEXTURE_2D, Some(source));
            gl.uniform_2_f32(
                Some(&self.upsample_prog.u_halfpixel),
                0.5 / input_width as f32,
                0.5 / input_height as f32,
            );
            gl.uniform_1_f32(
                Some(&self.upsample_prog.u_offset),
                params.radius * (1_i32 << index) as f32,
            );
            gl.uniform_2_f32(
                Some(&self.upsample_prog.u_scale_uv),
                target_scale.0,
                target_scale.1,
            );

            gl.enable_vertex_attrib_array(self.upsample_prog.a_position);
            gl.vertex_attrib_pointer_f32(
                self.upsample_prog.a_position,
                2,
                glow::FLOAT,
                false,
                8,
                0,
            );
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
            gl.disable_vertex_attrib_array(self.upsample_prog.a_position);
        }
    }

    /// Composites the blurred result over the damaged part of the region.
    ///
    /// # Safety
    ///
    /// Must be invoked while the OpenGL/EGL context is current.
    #[allow(clippy::too_many_arguments)]
    unsafe fn composite(
        &mut self,
        gl: &glow::Context,
        saved_fbo: Option<glow::NativeFramebuffer>,
        region: GlRect,
        dirty: GlRect,
        corner_radius: f32,
        params: &BlurParams,
        cache: &BlurCache,
    ) {
        let (x0, y0, region_w, region_h) = region;

        gl.bind_framebuffer(glow::FRAMEBUFFER, saved_fbo);
        gl.viewport(x0, y0, region_w, region_h);

        // Only the damaged part of the region may be written. Everything else
        // still holds the result of the previous frame, overwriting it would
        // destroy the content of the elements in front of the blur.
        gl.enable(glow::SCISSOR_TEST);
        gl.scissor(dirty.0, dirty.1, dirty.2, dirty.3);

        gl.enable(glow::BLEND);
        gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);

        gl.use_program(Some(self.composite_prog.program));
        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(cache.blurred));
        gl.uniform_1_i32(Some(&self.composite_prog.u_texture), 0);
        gl.uniform_2_f32(
            Some(&self.composite_prog.u_rect_size),
            region_w as f32,
            region_h as f32,
        );
        gl.uniform_1_f32(Some(&self.composite_prog.u_radius), corner_radius);
        gl.uniform_1_f32(Some(&self.composite_prog.u_opacity), 1.0);
        gl.uniform_4_f32(
            Some(&self.composite_prog.u_tint),
            params.tint[0],
            params.tint[1],
            params.tint[2],
            params.tint[3],
        );
        gl.uniform_1_f32(Some(&self.composite_prog.u_noise), params.noise);
        gl.uniform_1_f32(Some(&self.composite_prog.u_brightness), params.brightness);
        gl.uniform_1_f32(Some(&self.composite_prog.u_contrast), params.contrast);
        // The result texture holds the region at its full extent.
        gl.uniform_2_f32(Some(&self.composite_prog.u_scale_uv), 1.0, 1.0);

        gl.enable_vertex_attrib_array(self.composite_prog.a_position);
        gl.vertex_attrib_pointer_f32(self.composite_prog.a_position, 2, glow::FLOAT, false, 8, 0);
        gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        gl.disable_vertex_attrib_array(self.composite_prog.a_position);
    }
}

/// Computes the part of the region that has to be recomputed.
///
/// The damage is given in absolute physical coordinates and is projected into
/// framebuffer coordinates the same way the region itself is projected.
fn dirty_region(
    damage: &[Rectangle<i32, Physical>],
    projection: &[f32; 9],
    screen_w: i32,
    screen_h: i32,
    region: GlRect,
) -> Option<GlRect> {
    let (rx, ry, rw, rh) = region;

    let mut accumulated: Option<GlRect> = None;

    for rect in damage {
        let (dx, dy, dw, dh) = project_rect_to_gl(*rect, projection, screen_w, screen_h);
        if dw <= 0 || dh <= 0 {
            continue;
        }

        accumulated = Some(match accumulated {
            None => (dx, dy, dw, dh),
            Some((ax, ay, aw, ah)) => {
                let x = ax.min(dx);
                let y = ay.min(dy);
                let x2 = (ax + aw).max(dx + dw);
                let y2 = (ay + ah).max(dy + dh);
                (x, y, x2 - x, y2 - y)
            }
        });
    }

    let (ax, ay, aw, ah) = accumulated?;
    let x = ax.max(rx);
    let y = ay.max(ry);
    let x2 = (ax + aw).min(rx + rw);
    let y2 = (ay + ah).min(ry + rh);

    if x2 <= x || y2 <= y {
        return None;
    }

    Some((x, y, x2 - x, y2 - y))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Identity projection for a 1920x1080 display, matching the layout of the
    /// tests in `projection.rs`.
    fn projection() -> [f32; 9] {
        let mut proj = [0.0f32; 9];
        proj[0] = 2.0 / 1920.0;
        proj[4] = 2.0 / 1080.0;
        proj[6] = -1.0;
        proj[7] = -1.0;
        proj[8] = 1.0;
        proj
    }

    #[test]
    fn test_dirty_region_is_clipped_to_the_element_region() {
        let proj = projection();
        let region = (100, 200, 400, 300);

        // A damaged area that is fully inside the region.
        let damage = [Rectangle::new((150, 250).into(), (100, 100).into())];
        let dirty = dirty_region(&damage, &proj, 1920, 1080, region).unwrap();
        assert_eq!(dirty, (150, 250, 100, 100));

        // A damaged area that reaches outside of the region is clipped.
        let damage = [Rectangle::new((0, 0).into(), (200, 400).into())];
        let dirty = dirty_region(&damage, &proj, 1920, 1080, region).unwrap();
        assert_eq!(dirty, (100, 200, 100, 200));
    }

    #[test]
    fn test_dirty_region_is_empty_without_damage() {
        let proj = projection();
        let region = (100, 200, 400, 300);

        // Nothing damaged at all.
        assert!(dirty_region(&[], &proj, 1920, 1080, region).is_none());

        // Only damaged outside of the region.
        let damage = [Rectangle::new((1000, 1000).into(), (50, 50).into())];
        assert!(dirty_region(&damage, &proj, 1920, 1080, region).is_none());
    }

    #[test]
    fn test_dirty_region_unions_multiple_damage_areas() {
        let proj = projection();
        let region = (100, 200, 400, 300);

        let damage = [
            Rectangle::new((120, 220).into(), (50, 50).into()),
            Rectangle::new((300, 400).into(), (50, 50).into()),
        ];
        let dirty = dirty_region(&damage, &proj, 1920, 1080, region).unwrap();
        assert_eq!(dirty, (120, 220, 230, 230));
    }
}
