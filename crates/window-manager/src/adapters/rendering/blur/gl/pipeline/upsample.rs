//! Dual Kawase upsample pass.

use common::constants::MAX_BLUR_PASSES;
use glow::HasContext;

use crate::adapters::rendering::blur::config::BlurParams;

use super::cache::BlurCache;
use super::BlurPipeline;

/// Upsamples the pass targets back into the per-element result texture.
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
