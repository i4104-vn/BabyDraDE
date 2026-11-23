//! Composite pass: blends the blurred result over the damaged region.

use glow::HasContext;

use crate::adapters::rendering::blur::config::BlurParams;

use super::cache::BlurCache;
use super::setup::GlRect;
use super::BlurPipeline;

/// Composites the blurred result over the damaged part of the region.
///
/// # Safety
///
/// Must be invoked while the OpenGL/EGL context is current.
#[allow(clippy::too_many_arguments)]
// SAFETY: GL context is current and object names come from this pipeline.
pub(super) unsafe fn run(
    gl: &glow::Context,
    pipeline: &BlurPipeline,
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

    gl.use_program(Some(pipeline.composite_prog.program));
    gl.active_texture(glow::TEXTURE0);
    gl.bind_texture(glow::TEXTURE_2D, Some(cache.blurred));
    gl.uniform_1_i32(Some(&pipeline.composite_prog.u_texture), 0);
    gl.uniform_2_f32(
        Some(&pipeline.composite_prog.u_rect_size),
        region_w as f32,
        region_h as f32,
    );
    gl.uniform_1_f32(Some(&pipeline.composite_prog.u_radius), corner_radius);
    gl.uniform_1_f32(Some(&pipeline.composite_prog.u_opacity), 1.0);
    gl.uniform_4_f32(
        Some(&pipeline.composite_prog.u_tint),
        params.tint[0],
        params.tint[1],
        params.tint[2],
        params.tint[3],
    );
    gl.uniform_1_f32(Some(&pipeline.composite_prog.u_noise), params.noise);
    gl.uniform_1_f32(
        Some(&pipeline.composite_prog.u_brightness),
        params.brightness,
    );
    gl.uniform_1_f32(Some(&pipeline.composite_prog.u_contrast), params.contrast);
    // The result texture holds the region at its full extent.
    gl.uniform_2_f32(Some(&pipeline.composite_prog.u_scale_uv), 1.0, 1.0);

    gl.enable_vertex_attrib_array(pipeline.composite_prog.a_position);
    gl.vertex_attrib_pointer_f32(
        pipeline.composite_prog.a_position,
        2,
        glow::FLOAT,
        false,
        8,
        0,
    );
    gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
    gl.disable_vertex_attrib_array(pipeline.composite_prog.a_position);
}
