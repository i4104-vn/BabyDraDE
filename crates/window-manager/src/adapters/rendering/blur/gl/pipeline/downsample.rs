//! Dual Kawase downsample pass.

use common::constants::MAX_BLUR_PASSES;
use glow::HasContext;

use crate::adapters::rendering::blur::config::BlurParams;

use super::cache::BlurCache;
use super::BlurPipeline;

/// Downsamples the captured backdrop into progressively smaller pass targets.
///
/// # Safety
///
/// Must be invoked while the OpenGL/EGL context is current.
pub(super) unsafe fn run(
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
