//! Per-element blur texture cache model.

use glow::HasContext;
use smithay::utils::{Physical, Rectangle};

use super::super::shaders::setup_tex_params;

/// Cached state of a single blur element.
///
/// The blur samples the pixels that are drawn *below* the element. Those pixels
/// live in the output framebuffer and cannot be sampled directly, so they are
/// captured into `backdrop` and kept up to date with the damaged parts only.
/// `blurred` holds the result of the dual kawase pass, so that the element can
/// re-composite it without running the whole blur again.
pub(crate) struct BlurCache {
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
    // SAFETY: GL context is current and object names come from this pipeline.
    pub(crate) unsafe fn new(gl: &glow::Context) -> Result<Self, String> {
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
    // SAFETY: GL context is current and object names come from this pipeline.
    pub(crate) unsafe fn release(&mut self, gl: &glow::Context) {
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
    // SAFETY: GL context is current and object names come from this pipeline.
    pub(crate) unsafe fn resize(&mut self, gl: &glow::Context, width: i32, height: i32) -> bool {
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
        self.bytes = width as usize * height as usize * 4
            + result_width as usize * result_height as usize * 4;
        if !complete {
            self.failed_size = Some((width, height));
            return false;
        }

        self.failed_size = None;
        self.size = (width, height);
        true
    }
}
