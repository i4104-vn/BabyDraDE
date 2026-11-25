//! Dual Kawase blur render passes.

use glow::HasContext;
use common::constants::MAX_BLUR_PASSES;

use super::super::config::BlurParams;
use super::{BlurCache, BlurPipeline};

pub(crate) struct BlurPassTarget {
    pub(crate) texture: glow::Texture,
    pub(crate) framebuffer: glow::Framebuffer,
}

/// A rectangle in framebuffer coordinates.
pub(crate) type GlRect = (i32, i32, i32, i32);

/// Downsamples the captured backdrop into progressively smaller pass targets.
///
/// # Safety
///
/// Must be invoked while the OpenGL/EGL context is current.
pub(crate) unsafe fn run_downsample(
    gl: &glow::Context,
    pipeline: &BlurPipeline,
    cache: &mut BlurCache,
    region_w: i32,
    region_h: i32,
    params: &BlurParams,
) {
    let passes = params.passes.clamp(1, MAX_BLUR_PASSES);
    let backdrop_scale = (1.0f32, 1.0f32);
    let target_scale = (
        region_w as f32 / pipeline.allocated_w as f32,
        region_h as f32 / pipeline.allocated_h as f32,
    );

    // 1. Downsample into progressively smaller cached targets.
    gl.disable(glow::BLEND);
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(pipeline.vbo));

    gl.use_program(Some(pipeline.downsample_prog.program));
    gl.active_texture(glow::TEXTURE0);
    gl.uniform_1_i32(Some(&pipeline.downsample_prog.u_texture), 0);
    gl.enable_vertex_attrib_array(pipeline.downsample_prog.a_position);
    gl.vertex_attrib_pointer_f32(
        pipeline.downsample_prog.a_position,
        2,
        glow::FLOAT,
        false,
        8,
        0,
    );

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
            pipeline.pass_targets[index - 1].texture
        };
        let target_fbo = if is_final {
            cache.blurred_fbo
        } else {
            pipeline.pass_targets[index].framebuffer
        };

        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(target_fbo));
        gl.viewport(0, 0, target_width, target_height);
        gl.bind_texture(glow::TEXTURE_2D, Some(input_texture));
        gl.uniform_2_f32(
            Some(&pipeline.downsample_prog.u_halfpixel),
            0.5 / input_w as f32,
            0.5 / input_h as f32,
        );
        gl.uniform_1_f32(
            Some(&pipeline.downsample_prog.u_offset),
            params.radius * (1.0 + index as f32 * 0.5),
        );
        gl.uniform_2_f32(
            Some(&pipeline.downsample_prog.u_scale_uv),
            input_scale.0,
            input_scale.1,
        );
        gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);

        input_w = (pipeline.allocated_w / (1_i32 << (index + 1))).max(1);
        input_h = (pipeline.allocated_h / (1_i32 << (index + 1))).max(1);
        input_scale = target_scale;
    }
    gl.disable_vertex_attrib_array(pipeline.downsample_prog.a_position);
}
/// Upsamples the pass targets back into the per-element result texture.
///
/// # Safety
///
/// Must be invoked while the OpenGL/EGL context is current.
pub(crate) unsafe fn run_upsample(
    gl: &glow::Context,
    pipeline: &BlurPipeline,
    cache: &mut BlurCache,
    region_w: i32,
    region_h: i32,
    params: &BlurParams,
) {
    let passes = params.passes.clamp(1, MAX_BLUR_PASSES);
    let target_scale = (
        region_w as f32 / pipeline.allocated_w as f32,
        region_h as f32 / pipeline.allocated_h as f32,
    );

    // 2. Upsample back to the first target. Each pass reuses the next target.
    gl.use_program(Some(pipeline.upsample_prog.program));
    gl.uniform_1_i32(Some(&pipeline.upsample_prog.u_texture), 0);
    for index in (0..passes.saturating_sub(1)).rev() {
        // The last upsample step writes into the per element result texture.
        let is_final = index == 0;

        let (source, target_fbo) = if is_final {
            (pipeline.pass_targets[index + 1].texture, cache.blurred_fbo)
        } else {
            (
                pipeline.pass_targets[index + 1].texture,
                pipeline.pass_targets[index].framebuffer,
            )
        };

        let divisor = 1_i32 << (index + 1);
        let target_width = (region_w / divisor).max(1);
        let target_height = (region_h / divisor).max(1);
        let input_divisor = 1_i32 << (index + 2);
        let input_width = (pipeline.allocated_w / input_divisor).max(1);
        let input_height = (pipeline.allocated_h / input_divisor).max(1);

        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(target_fbo));
        gl.viewport(0, 0, target_width, target_height);
        gl.bind_texture(glow::TEXTURE_2D, Some(source));
        gl.uniform_2_f32(
            Some(&pipeline.upsample_prog.u_halfpixel),
            0.5 / input_width as f32,
            0.5 / input_height as f32,
        );
        gl.uniform_1_f32(
            Some(&pipeline.upsample_prog.u_offset),
            params.radius * (1.0 + index as f32 * 0.5),
        );
        gl.uniform_2_f32(
            Some(&pipeline.upsample_prog.u_scale_uv),
            target_scale.0,
            target_scale.1,
        );

        gl.enable_vertex_attrib_array(pipeline.upsample_prog.a_position);
        gl.vertex_attrib_pointer_f32(
            pipeline.upsample_prog.a_position,
            2,
            glow::FLOAT,
            false,
            8,
            0,
        );
        gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        gl.disable_vertex_attrib_array(pipeline.upsample_prog.a_position);
    }
}
/// Runs the dual kawase pass on the cached backdrop and stores the result in
/// the cache, so that it can be re-composited later on.
///
/// # Safety
///
/// Must be invoked while the OpenGL/EGL context is current.
pub(crate) unsafe fn run_blur(
    gl: &glow::Context,
    pipeline: &BlurPipeline,
    region_w: i32,
    region_h: i32,
    params: &BlurParams,
    cache: &mut BlurCache,
) {
    run_downsample(gl, pipeline, cache, region_w, region_h, params);
    run_upsample(gl, pipeline, cache, region_w, region_h, params);
}
/// Composites the blurred result over the damaged part of the region.
///
/// # Safety
///
/// Must be invoked while the OpenGL/EGL context is current.
#[allow(clippy::too_many_arguments)]
// SAFETY: GL context is current and object names come from this pipeline.
pub(crate) unsafe fn run_composite(
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
