//! GL object setup: shader programs, quad buffer and pass render targets.

use glow::HasContext;

use common::constants::MAX_BLUR_PASSES;

use super::super::shaders::{
    compile_composite, compile_downsample, compile_upsample, setup_tex_params, QUAD_VERTICES,
};
use super::BlurPipeline;

pub(super) struct BlurPassTarget {
    pub(super) texture: glow::Texture,
    pub(super) framebuffer: glow::Framebuffer,
}

/// A rectangle in framebuffer coordinates.
pub(super) type GlRect = (i32, i32, i32, i32);

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
