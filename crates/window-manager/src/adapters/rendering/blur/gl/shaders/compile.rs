//! GLSL shader compilation and program linking.

use glow::HasContext;

use super::programs::{CompositeProgram, DownsampleProgram, UpsampleProgram};
use super::sources::{
    COMPOSITE_SHADER_SRC, DOWNSAMPLE_SHADER_SRC, UPSAMPLE_SHADER_SRC, VERTEX_SHADER_SRC,
};

// SAFETY: GL context is current and object names come from this pipeline.
pub(crate) unsafe fn setup_tex_params(gl: &glow::Context) {
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MIN_FILTER,
        glow::LINEAR as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MAG_FILTER,
        glow::LINEAR as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_WRAP_S,
        glow::CLAMP_TO_EDGE as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_WRAP_T,
        glow::CLAMP_TO_EDGE as i32,
    );
}

// SAFETY: GL context is current and object names come from this pipeline.
unsafe fn compile_shader(
    gl: &glow::Context,
    shader_type: u32,
    source: &str,
) -> Result<glow::Shader, String> {
    let shader = gl.create_shader(shader_type)?;
    gl.shader_source(shader, source);
    gl.compile_shader(shader);
    if !gl.get_shader_compile_status(shader) {
        let log = gl.get_shader_info_log(shader);
        gl.delete_shader(shader);
        return Err(format!("Shader compilation failed: {log}"));
    }
    Ok(shader)
}

// SAFETY: GL context is current and object names come from this pipeline.
unsafe fn link_program(
    gl: &glow::Context,
    vs_src: &str,
    fs_src: &str,
) -> Result<glow::Program, String> {
    let vs = compile_shader(gl, glow::VERTEX_SHADER, vs_src)?;
    let fs = compile_shader(gl, glow::FRAGMENT_SHADER, fs_src)?;
    let prog = gl.create_program()?;
    gl.attach_shader(prog, vs);
    gl.attach_shader(prog, fs);
    let position_location = (gl.get_parameter_i32(glow::MAX_VERTEX_ATTRIBS).max(1) - 1) as u32;
    gl.bind_attrib_location(prog, position_location, "a_position");
    gl.link_program(prog);

    gl.delete_shader(vs);
    gl.delete_shader(fs);

    if !gl.get_program_link_status(prog) {
        let log = gl.get_program_info_log(prog);
        gl.delete_program(prog);
        return Err(format!("Program linking failed: {log}"));
    }
    Ok(prog)
}

// SAFETY: GL context is current and object names come from this pipeline.
pub(crate) unsafe fn compile_downsample(gl: &glow::Context) -> Result<DownsampleProgram, String> {
    let prog = link_program(gl, VERTEX_SHADER_SRC, DOWNSAMPLE_SHADER_SRC)?;
    let u_texture = gl
        .get_uniform_location(prog, "u_texture")
        .ok_or("u_texture not found")?;
    let u_halfpixel = gl
        .get_uniform_location(prog, "u_halfpixel")
        .ok_or("u_halfpixel not found")?;
    let u_offset = gl
        .get_uniform_location(prog, "u_offset")
        .ok_or("u_offset not found")?;
    let u_scale_uv = gl
        .get_uniform_location(prog, "u_scale_uv")
        .ok_or("u_scale_uv not found")?;
    let a_position = gl
        .get_attrib_location(prog, "a_position")
        .ok_or("a_position not found")?;

    Ok(DownsampleProgram {
        program: prog,
        u_texture,
        u_halfpixel,
        u_offset,
        u_scale_uv,
        a_position,
    })
}

// SAFETY: GL context is current and object names come from this pipeline.
pub(crate) unsafe fn compile_upsample(gl: &glow::Context) -> Result<UpsampleProgram, String> {
    let prog = link_program(gl, VERTEX_SHADER_SRC, UPSAMPLE_SHADER_SRC)?;
    let u_texture = gl
        .get_uniform_location(prog, "u_texture")
        .ok_or("u_texture not found")?;
    let u_halfpixel = gl
        .get_uniform_location(prog, "u_halfpixel")
        .ok_or("u_halfpixel not found")?;
    let u_offset = gl
        .get_uniform_location(prog, "u_offset")
        .ok_or("u_offset not found")?;
    let u_scale_uv = gl
        .get_uniform_location(prog, "u_scale_uv")
        .ok_or("u_scale_uv not found")?;
    let a_position = gl
        .get_attrib_location(prog, "a_position")
        .ok_or("a_position not found")?;

    Ok(UpsampleProgram {
        program: prog,
        u_texture,
        u_halfpixel,
        u_offset,
        u_scale_uv,
        a_position,
    })
}

// SAFETY: GL context is current and object names come from this pipeline.
pub(crate) unsafe fn compile_composite(gl: &glow::Context) -> Result<CompositeProgram, String> {
    let prog = link_program(gl, VERTEX_SHADER_SRC, COMPOSITE_SHADER_SRC)?;
    let u_texture = gl
        .get_uniform_location(prog, "u_texture")
        .ok_or("u_texture not found")?;
    let u_rect_size = gl
        .get_uniform_location(prog, "u_rect_size")
        .ok_or("u_rect_size not found")?;
    let u_radius = gl
        .get_uniform_location(prog, "u_radius")
        .ok_or("u_radius not found")?;
    let u_opacity = gl
        .get_uniform_location(prog, "u_opacity")
        .ok_or("u_opacity not found")?;
    let u_tint = gl
        .get_uniform_location(prog, "u_tint")
        .ok_or("u_tint not found")?;
    let u_noise = gl
        .get_uniform_location(prog, "u_noise")
        .ok_or("u_noise not found")?;
    let u_brightness = gl
        .get_uniform_location(prog, "u_brightness")
        .ok_or("u_brightness not found")?;
    let u_contrast = gl
        .get_uniform_location(prog, "u_contrast")
        .ok_or("u_contrast not found")?;
    let u_scale_uv = gl
        .get_uniform_location(prog, "u_scale_uv")
        .ok_or("u_scale_uv not found")?;
    let a_position = gl
        .get_attrib_location(prog, "a_position")
        .ok_or("a_position not found")?;

    Ok(CompositeProgram {
        program: prog,
        u_texture,
        u_rect_size,
        u_radius,
        u_opacity,
        u_tint,
        u_noise,
        u_brightness,
        u_contrast,
        u_scale_uv,
        a_position,
    })
}
