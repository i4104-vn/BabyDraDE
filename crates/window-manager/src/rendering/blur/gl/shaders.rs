use glow::HasContext;

pub(crate) const QUAD_VERTICES: [f32; 8] = [-1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0];

const VERTEX_SHADER_SRC: &str = r#"
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif

attribute vec2 a_position;
varying vec2 v_tex_coord;
varying vec2 v_norm_coord;
uniform vec2 u_scale_uv;

void main() {
    v_norm_coord = a_position * 0.5 + 0.5;
    v_tex_coord = v_norm_coord * u_scale_uv;
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#;

const DOWNSAMPLE_SHADER_SRC: &str = r#"
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif

varying vec2 v_tex_coord;
uniform sampler2D u_texture;
uniform vec2 u_halfpixel;
uniform float u_offset;
uniform vec2 u_scale_uv;

vec4 sample_clamped(vec2 uv) {
    return texture2D(u_texture, clamp(uv, vec2(0.0), u_scale_uv));
}

void main() {
    vec2 d = u_halfpixel * u_offset;
    vec4 sum = sample_clamped(v_tex_coord) * 4.0;
    sum += sample_clamped(v_tex_coord - d);
    sum += sample_clamped(v_tex_coord + d);
    sum += sample_clamped(v_tex_coord + vec2(d.x, -d.y));
    sum += sample_clamped(v_tex_coord + vec2(-d.x, d.y));
    gl_FragColor = sum / 8.0;
}
"#;

const UPSAMPLE_SHADER_SRC: &str = r#"
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif

varying vec2 v_tex_coord;
uniform sampler2D u_texture;
uniform vec2 u_halfpixel;
uniform float u_offset;
uniform vec2 u_scale_uv;

vec4 sample_clamped(vec2 uv) {
    return texture2D(u_texture, clamp(uv, vec2(0.0), u_scale_uv));
}

void main() {
    vec2 d = u_halfpixel * u_offset;
    vec4 sum = vec4(0.0);
    sum += sample_clamped(v_tex_coord + vec2(-d.x * 2.0, 0.0));
    sum += sample_clamped(v_tex_coord + vec2(-d.x, d.y)) * 2.0;
    sum += sample_clamped(v_tex_coord + vec2(0.0, d.y * 2.0));
    sum += sample_clamped(v_tex_coord + vec2(d.x, d.y)) * 2.0;
    sum += sample_clamped(v_tex_coord + vec2(d.x * 2.0, 0.0));
    sum += sample_clamped(v_tex_coord + vec2(d.x, -d.y)) * 2.0;
    sum += sample_clamped(v_tex_coord + vec2(0.0, -d.y * 2.0));
    sum += sample_clamped(v_tex_coord + vec2(-d.x, -d.y)) * 2.0;
    sum += sample_clamped(v_tex_coord) * 4.0;
    gl_FragColor = sum / 16.0;
}
"#;

const COMPOSITE_SHADER_SRC: &str = r#"
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif

varying vec2 v_tex_coord;
varying vec2 v_norm_coord;
uniform sampler2D u_texture;
uniform vec2 u_rect_size;
uniform float u_radius;
uniform float u_opacity;
uniform vec4 u_tint;
uniform float u_noise;
uniform float u_brightness;
uniform float u_contrast;
uniform vec2 u_scale_uv;

float rounded_box_sdf(vec2 p, vec2 b, float r) {
    vec2 q = abs(p) - b + vec2(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r;
}

// Robust pseudorandom hash that avoids fp16 overflow on mobile/tablet GPUs
float rand(vec2 co) {
    vec2 p = fract(co * vec2(443.897, 441.423));
    p += dot(p, p.yx + 19.19);
    return fract((p.x + p.y) * p.x);
}

void main() {
    vec2 half_size = u_rect_size * 0.5;
    vec2 p = (v_norm_coord - 0.5) * u_rect_size;
    float r = min(u_radius, min(half_size.x, half_size.y));
    float d = rounded_box_sdf(p, half_size, r);
    if (d > 1.0) {
        discard;
    }
    float alpha = clamp(0.5 - d, 0.0, 1.0) * u_opacity;

    vec2 uv = clamp(v_tex_coord, vec2(0.0), u_scale_uv);
    vec4 color = texture2D(u_texture, uv);
    color.rgb = (color.rgb - 0.5) * u_contrast + 0.5;
    color.rgb *= u_brightness;
    color.rgb = mix(color.rgb, u_tint.rgb, u_tint.a);

    float n = (rand(v_norm_coord * u_rect_size) - 0.5) * u_noise;
    color.rgb += vec3(n);

    gl_FragColor = vec4(color.rgb * alpha, alpha);
}
"#;

pub(crate) struct DownsampleProgram {
    pub(crate) program: glow::Program,
    pub(crate) u_texture: glow::UniformLocation,
    pub(crate) u_halfpixel: glow::UniformLocation,
    pub(crate) u_offset: glow::UniformLocation,
    pub(crate) u_scale_uv: glow::UniformLocation,
    pub(crate) a_position: u32,
}

pub(crate) struct UpsampleProgram {
    pub(crate) program: glow::Program,
    pub(crate) u_texture: glow::UniformLocation,
    pub(crate) u_halfpixel: glow::UniformLocation,
    pub(crate) u_offset: glow::UniformLocation,
    pub(crate) u_scale_uv: glow::UniformLocation,
    pub(crate) a_position: u32,
}

pub(crate) struct CompositeProgram {
    pub(crate) program: glow::Program,
    pub(crate) u_texture: glow::UniformLocation,
    pub(crate) u_rect_size: glow::UniformLocation,
    pub(crate) u_radius: glow::UniformLocation,
    pub(crate) u_opacity: glow::UniformLocation,
    pub(crate) u_tint: glow::UniformLocation,
    pub(crate) u_noise: glow::UniformLocation,
    pub(crate) u_brightness: glow::UniformLocation,
    pub(crate) u_contrast: glow::UniformLocation,
    pub(crate) u_scale_uv: glow::UniformLocation,
    pub(crate) a_position: u32,
}

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
