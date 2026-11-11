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

struct BlurPassTarget {
    texture: glow::Texture,
    framebuffer: glow::Framebuffer,
}

pub struct BlurPipeline {
    downsample_prog: DownsampleProgram,
    upsample_prog: UpsampleProgram,
    composite_prog: CompositeProgram,
    vbo: glow::Buffer,

    capture_tex: glow::Texture,
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

        let capture_tex = gl.create_texture()?;
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
            capture_tex,
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

        // 1. Capture texture (full size)
        gl.bind_texture(glow::TEXTURE_2D, Some(self.capture_tex));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            new_w,
            new_h,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(None),
        );
        setup_tex_params(gl);

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

    /// Renders the Dual Kawase blur backdrop directly over `dst`, taking into
    /// account screen rotation/orientation and GL scissor states.
    ///
    /// # Safety
    ///
    /// Must be invoked while the OpenGL/EGL context is current.
    pub unsafe fn render_blur(
        &mut self,
        gl: &glow::Context,
        dst: Rectangle<i32, Physical>,
        corner_radius: f32,
        projection: [f32; 9],
        params: &BlurParams,
    ) {
        if dst.size.w <= 8 || dst.size.h <= 8 || !params.enabled {
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
            return;
        }

        // Offscreen passes must not inherit the compositor's output damage scissor.
        gl.disable(glow::SCISSOR_TEST);

        // Project dst rectangle to physical OpenGL framebuffer coordinates using Smithay's exact frame projection
        let (raw_gl_x, raw_gl_y, raw_gl_w, raw_gl_h) =
            project_rect_to_gl(dst, &projection, screen_w, screen_h);

        // Clip against physical framebuffer bounds
        let gl_x_clamped = raw_gl_x.max(0);
        let gl_y_clamped = raw_gl_y.max(0);
        let gl_x2 = (raw_gl_x + raw_gl_w).min(screen_w);
        let gl_y2 = (raw_gl_y + raw_gl_h).min(screen_h);
        let gl_w_clamped = gl_x2 - gl_x_clamped;
        let gl_h_clamped = gl_y2 - gl_y_clamped;

        if gl_w_clamped <= 8 || gl_h_clamped <= 8 {
            return;
        }

        let gl_x = gl_x_clamped;
        let gl_y = gl_y_clamped;
        let gl_w = gl_w_clamped;
        let gl_h = gl_h_clamped;

        if gl_w <= 8 || gl_h <= 8 {
            return;
        }

        gl.active_texture(glow::TEXTURE0);
        // Do not composite an uninitialized texture if the driver rejects one
        // of the offscreen targets. The normal surface render must remain
        // visible instead of being covered by a black quad.
        if !self.ensure_texture_sizes(gl, gl_w, gl_h) {
            return;
        }

        let scale_u = gl_w as f32 / self.allocated_w as f32;
        let scale_v = gl_h as f32 / self.allocated_h as f32;

        // 1. Capture the background behind the window from the active framebuffer into capture_tex
        gl.bind_texture(glow::TEXTURE_2D, Some(self.capture_tex));
        gl.copy_tex_sub_image_2d(glow::TEXTURE_2D, 0, 0, 0, gl_x, gl_y, gl_w, gl_h);

        gl.disable(glow::BLEND);
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));

        let passes = params.passes.clamp(1, MAX_BLUR_PASSES);

        // 2. Downsample into progressively smaller cached targets.
        gl.use_program(Some(self.downsample_prog.program));
        gl.active_texture(glow::TEXTURE0);
        gl.uniform_1_i32(Some(&self.downsample_prog.u_texture), 0);
        gl.enable_vertex_attrib_array(self.downsample_prog.a_position);
        gl.vertex_attrib_pointer_f32(self.downsample_prog.a_position, 2, glow::FLOAT, false, 8, 0);

        let mut input_width = self.allocated_w;
        let mut input_height = self.allocated_h;
        for index in 0..passes {
            let target = &self.pass_targets[index];
            let divisor = 1_i32 << (index + 1);
            let target_width = (gl_w / divisor).max(1);
            let target_height = (gl_h / divisor).max(1);
            let input_texture = if index == 0 {
                self.capture_tex
            } else {
                self.pass_targets[index - 1].texture
            };

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(target.framebuffer));
            gl.viewport(0, 0, target_width, target_height);
            gl.bind_texture(glow::TEXTURE_2D, Some(input_texture));
            gl.uniform_2_f32(
                Some(&self.downsample_prog.u_halfpixel),
                0.5 / input_width as f32,
                0.5 / input_height as f32,
            );
            gl.uniform_1_f32(
                Some(&self.downsample_prog.u_offset),
                params.radius * (1_i32 << index) as f32 * 0.5,
            );
            gl.uniform_2_f32(Some(&self.downsample_prog.u_scale_uv), scale_u, scale_v);
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);

            input_width = (input_width / 2).max(1);
            input_height = (input_height / 2).max(1);
        }
        gl.disable_vertex_attrib_array(self.downsample_prog.a_position);

        // 3. Upsample back to the first target. Each pass reuses the next target.
        gl.use_program(Some(self.upsample_prog.program));
        gl.uniform_1_i32(Some(&self.upsample_prog.u_texture), 0);
        for index in (0..passes.saturating_sub(1)).rev() {
            let target = &self.pass_targets[index];
            let source = &self.pass_targets[index + 1];
            let divisor = 1_i32 << (index + 1);
            let target_width = (gl_w / divisor).max(1);
            let target_height = (gl_h / divisor).max(1);
            let input_divisor = 1_i32 << (index + 2);
            let input_width = (self.allocated_w / input_divisor).max(1);
            let input_height = (self.allocated_h / input_divisor).max(1);

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(target.framebuffer));
            gl.viewport(0, 0, target_width, target_height);
            gl.bind_texture(glow::TEXTURE_2D, Some(source.texture));
            gl.uniform_2_f32(
                Some(&self.upsample_prog.u_halfpixel),
                0.5 / input_width as f32,
                0.5 / input_height as f32,
            );
            gl.uniform_1_f32(
                Some(&self.upsample_prog.u_offset),
                params.radius * (1_i32 << index) as f32,
            );
            gl.uniform_2_f32(Some(&self.upsample_prog.u_scale_uv), scale_u, scale_v);

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

        // 5. Final Composite Pass: blurred tex1 -> screen framebuffer (within gl_x, gl_y, gl_w, gl_h)
        gl.bind_framebuffer(glow::FRAMEBUFFER, saved_fbo);
        gl.viewport(gl_x, gl_y, gl_w, gl_h);

        gl.enable(glow::BLEND);
        gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);

        gl.use_program(Some(self.composite_prog.program));
        gl.bind_texture(glow::TEXTURE_2D, Some(self.pass_targets[0].texture));
        gl.uniform_1_i32(Some(&self.composite_prog.u_texture), 0);
        gl.uniform_2_f32(
            Some(&self.composite_prog.u_rect_size),
            gl_w as f32,
            gl_h as f32,
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
        gl.uniform_2_f32(Some(&self.composite_prog.u_scale_uv), scale_u, scale_v);

        gl.enable_vertex_attrib_array(self.composite_prog.a_position);
        gl.vertex_attrib_pointer_f32(self.composite_prog.a_position, 2, glow::FLOAT, false, 8, 0);
        gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        gl.disable_vertex_attrib_array(self.composite_prog.a_position);

        drop(gl_state);
    }
}
