use glow::HasContext;
use smithay::utils::{Physical, Rectangle};

use super::config::BlurParams;

const QUAD_VERTICES: [f32; 8] = [
    -1.0, -1.0,
     1.0, -1.0,
    -1.0,  1.0,
     1.0,  1.0,
];

const VERTEX_SHADER_SRC: &str = r#"
attribute vec2 a_position;
varying vec2 v_tex_coord;

void main() {
    v_tex_coord = a_position * 0.5 + 0.5;
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#;

const DOWNSAMPLE_SHADER_SRC: &str = r#"
precision mediump float;
varying vec2 v_tex_coord;
uniform sampler2D u_texture;
uniform vec2 u_halfpixel;
uniform float u_offset;

void main() {
    vec2 d = u_halfpixel * u_offset;
    vec4 sum = texture2D(u_texture, v_tex_coord) * 4.0;
    sum += texture2D(u_texture, v_tex_coord - d);
    sum += texture2D(u_texture, v_tex_coord + d);
    sum += texture2D(u_texture, v_tex_coord + vec2(d.x, -d.y));
    sum += texture2D(u_texture, v_tex_coord + vec2(-d.x, d.y));
    gl_FragColor = sum / 8.0;
}
"#;

const UPSAMPLE_SHADER_SRC: &str = r#"
precision mediump float;
varying vec2 v_tex_coord;
uniform sampler2D u_texture;
uniform vec2 u_halfpixel;
uniform float u_offset;

void main() {
    vec2 d = u_halfpixel * u_offset;
    vec4 sum = vec4(0.0);
    sum += texture2D(u_texture, v_tex_coord + vec2(-d.x * 2.0, 0.0));
    sum += texture2D(u_texture, v_tex_coord + vec2(-d.x, d.y)) * 2.0;
    sum += texture2D(u_texture, v_tex_coord + vec2(0.0, d.y * 2.0));
    sum += texture2D(u_texture, v_tex_coord + vec2(d.x, d.y)) * 2.0;
    sum += texture2D(u_texture, v_tex_coord + vec2(d.x * 2.0, 0.0));
    sum += texture2D(u_texture, v_tex_coord + vec2(d.x, -d.y)) * 2.0;
    sum += texture2D(u_texture, v_tex_coord + vec2(0.0, -d.y * 2.0));
    sum += texture2D(u_texture, v_tex_coord + vec2(-d.x, -d.y)) * 2.0;
    sum += texture2D(u_texture, v_tex_coord) * 4.0;
    gl_FragColor = sum / 16.0;
}
"#;

const COMPOSITE_SHADER_SRC: &str = r#"
precision mediump float;
varying vec2 v_tex_coord;
uniform sampler2D u_texture;
uniform vec2 u_rect_size;
uniform float u_radius;
uniform float u_opacity;
uniform vec4 u_tint;
uniform float u_noise;
uniform float u_brightness;
uniform float u_contrast;

float rounded_box_sdf(vec2 p, vec2 b, float r) {
    vec2 q = abs(p) - b + vec2(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r;
}

float rand(vec2 co) {
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    vec2 half_size = u_rect_size * 0.5;
    vec2 p = (v_tex_coord - 0.5) * u_rect_size;
    float r = min(u_radius, min(half_size.x, half_size.y));
    float d = rounded_box_sdf(p, half_size, r);
    if (d > 1.0) {
        discard;
    }
    float alpha = clamp(0.5 - d, 0.0, 1.0) * u_opacity;

    vec4 color = texture2D(u_texture, v_tex_coord);
    color.rgb = (color.rgb - 0.5) * u_contrast + 0.5;
    color.rgb *= u_brightness;
    color.rgb = mix(color.rgb, u_tint.rgb, u_tint.a);

    float n = (rand(v_tex_coord * u_rect_size) - 0.5) * u_noise;
    color.rgb += vec3(n);

    gl_FragColor = vec4(color.rgb * alpha, alpha);
}
"#;

struct DownsampleProgram {
    program: glow::Program,
    u_texture: glow::UniformLocation,
    u_halfpixel: glow::UniformLocation,
    u_offset: glow::UniformLocation,
    a_position: u32,
}

struct UpsampleProgram {
    program: glow::Program,
    u_texture: glow::UniformLocation,
    u_halfpixel: glow::UniformLocation,
    u_offset: glow::UniformLocation,
    a_position: u32,
}

struct CompositeProgram {
    program: glow::Program,
    u_texture: glow::UniformLocation,
    u_rect_size: glow::UniformLocation,
    u_radius: glow::UniformLocation,
    u_opacity: glow::UniformLocation,
    u_tint: glow::UniformLocation,
    u_noise: glow::UniformLocation,
    u_brightness: glow::UniformLocation,
    u_contrast: glow::UniformLocation,
    a_position: u32,
}

pub struct BlurPipeline {
    downsample_prog: DownsampleProgram,
    upsample_prog: UpsampleProgram,
    composite_prog: CompositeProgram,
    vbo: glow::Buffer,

    capture_tex: glow::Texture,
    tex1: glow::Texture,
    fbo1: glow::Framebuffer,
    tex2: glow::Texture,
    fbo2: glow::Framebuffer,

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
        let tex1 = gl.create_texture()?;
        let fbo1 = gl.create_framebuffer()?;
        let tex2 = gl.create_texture()?;
        let fbo2 = gl.create_framebuffer()?;

        Ok(Self {
            downsample_prog,
            upsample_prog,
            composite_prog,
            vbo,
            capture_tex,
            tex1,
            fbo1,
            tex2,
            fbo2,
            allocated_w: 0,
            allocated_h: 0,
        })
    }

    unsafe fn ensure_texture_sizes(&mut self, gl: &glow::Context, w: i32, h: i32) {
        if self.allocated_w >= w && self.allocated_h >= h {
            return;
        }

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

        // 2. Pass 1 texture (half size)
        let w_half = (new_w / 2).max(1);
        let h_half = (new_h / 2).max(1);
        gl.bind_texture(glow::TEXTURE_2D, Some(self.tex1));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            w_half,
            h_half,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(None),
        );
        setup_tex_params(gl);
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo1));
        gl.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            glow::TEXTURE_2D,
            Some(self.tex1),
            0,
        );

        // 3. Pass 2 texture (quarter size)
        let w_quarter = (new_w / 4).max(1);
        let h_quarter = (new_h / 4).max(1);
        gl.bind_texture(glow::TEXTURE_2D, Some(self.tex2));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            w_quarter,
            h_quarter,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(None),
        );
        setup_tex_params(gl);
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo2));
        gl.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            glow::TEXTURE_2D,
            Some(self.tex2),
            0,
        );

        gl.bind_texture(glow::TEXTURE_2D, None);
        gl.bind_framebuffer(glow::FRAMEBUFFER, None);

        self.allocated_w = new_w;
        self.allocated_h = new_h;
    }

    /// Renders the Dual Kawase blur backdrop directly over `dst`
    ///
    /// # Safety
    ///
    /// Must be invoked while the OpenGL/EGL context is current.
    pub unsafe fn render_blur(
        &mut self,
        gl: &glow::Context,
        dst: Rectangle<i32, Physical>,
        corner_radius: f32,
        params: &BlurParams,
    ) {
        if dst.size.w <= 8 || dst.size.h <= 8 || !params.enabled {
            return;
        }

        // Save current GL state
        let saved_fbo = gl.get_parameter_i32(glow::FRAMEBUFFER_BINDING) as u32;
        let mut saved_viewport = [0i32; 4];
        gl.get_parameter_i32_slice(glow::VIEWPORT, &mut saved_viewport);

        let screen_w = saved_viewport[2];
        let screen_h = saved_viewport[3];

        if screen_w <= 0 || screen_h <= 0 {
            return;
        }

        // Convert physical Wayland top-left coords to GL bottom-left coords
        let gl_x = dst.loc.x.clamp(0, screen_w - 1);
        let gl_y = (screen_h - (dst.loc.y + dst.size.h)).clamp(0, screen_h - 1);
        let gl_w = dst.size.w.min(screen_w - gl_x);
        let gl_h = dst.size.h.min(screen_h - gl_y);

        if gl_w <= 8 || gl_h <= 8 {
            return;
        }

        self.ensure_texture_sizes(gl, gl_w, gl_h);

        // 1. Capture the background behind the window from the active framebuffer into capture_tex
        gl.bind_texture(glow::TEXTURE_2D, Some(self.capture_tex));
        gl.copy_tex_sub_image_2d(
            glow::TEXTURE_2D,
            0,
            0,
            0,
            gl_x,
            gl_y,
            gl_w,
            gl_h,
        );

        gl.disable(glow::BLEND);
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));

        let half_w = (gl_w / 2).max(1);
        let half_h = (gl_h / 2).max(1);
        let quarter_w = (gl_w / 4).max(1);
        let quarter_h = (gl_h / 4).max(1);

        // 2. Downsample Pass 1: capture_tex -> fbo1 (tex1, half-size)
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo1));
        gl.viewport(0, 0, half_w, half_h);
        gl.use_program(Some(self.downsample_prog.program));

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(self.capture_tex));
        gl.uniform_1_i32(Some(&self.downsample_prog.u_texture), 0);
        gl.uniform_2_f32(
            Some(&self.downsample_prog.u_halfpixel),
            0.5 / gl_w as f32,
            0.5 / gl_h as f32,
        );
        gl.uniform_1_f32(Some(&self.downsample_prog.u_offset), params.radius * 0.5);

        gl.enable_vertex_attrib_array(self.downsample_prog.a_position);
        gl.vertex_attrib_pointer_f32(self.downsample_prog.a_position, 2, glow::FLOAT, false, 8, 0);
        gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);

        // 3. Downsample Pass 2: tex1 -> fbo2 (tex2, quarter-size)
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo2));
        gl.viewport(0, 0, quarter_w, quarter_h);

        gl.bind_texture(glow::TEXTURE_2D, Some(self.tex1));
        gl.uniform_2_f32(
            Some(&self.downsample_prog.u_halfpixel),
            1.0 / gl_w as f32,
            1.0 / gl_h as f32,
        );
        gl.uniform_1_f32(Some(&self.downsample_prog.u_offset), params.radius * 1.0);
        gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        gl.disable_vertex_attrib_array(self.downsample_prog.a_position);

        // 4. Upsample Pass: tex2 -> fbo1 (tex1, half-size)
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo1));
        gl.viewport(0, 0, half_w, half_h);
        gl.use_program(Some(self.upsample_prog.program));

        gl.bind_texture(glow::TEXTURE_2D, Some(self.tex2));
        gl.uniform_1_i32(Some(&self.upsample_prog.u_texture), 0);
        gl.uniform_2_f32(
            Some(&self.upsample_prog.u_halfpixel),
            2.0 / gl_w as f32,
            2.0 / gl_h as f32,
        );
        gl.uniform_1_f32(Some(&self.upsample_prog.u_offset), params.radius * 1.0);

        gl.enable_vertex_attrib_array(self.upsample_prog.a_position);
        gl.vertex_attrib_pointer_f32(self.upsample_prog.a_position, 2, glow::FLOAT, false, 8, 0);
        gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        gl.disable_vertex_attrib_array(self.upsample_prog.a_position);

        // 5. Final Composite Pass: blurred tex1 -> screen framebuffer (within gl_x, gl_y, gl_w, gl_h)
        let target_fbo = std::num::NonZeroU32::new(saved_fbo).map(glow::NativeFramebuffer);
        gl.bind_framebuffer(glow::FRAMEBUFFER, target_fbo);
        gl.viewport(gl_x, gl_y, gl_w, gl_h);

        gl.enable(glow::BLEND);
        gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);

        gl.use_program(Some(self.composite_prog.program));
        gl.bind_texture(glow::TEXTURE_2D, Some(self.tex1));
        gl.uniform_1_i32(Some(&self.composite_prog.u_texture), 0);
        gl.uniform_2_f32(Some(&self.composite_prog.u_rect_size), gl_w as f32, gl_h as f32);
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

        gl.enable_vertex_attrib_array(self.composite_prog.a_position);
        gl.vertex_attrib_pointer_f32(self.composite_prog.a_position, 2, glow::FLOAT, false, 8, 0);
        gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        gl.disable_vertex_attrib_array(self.composite_prog.a_position);

        // Restore saved GL states
        gl.bind_buffer(glow::ARRAY_BUFFER, None);
        gl.bind_texture(glow::TEXTURE_2D, None);
        gl.use_program(None);
        gl.viewport(saved_viewport[0], saved_viewport[1], saved_viewport[2], saved_viewport[3]);
        gl.enable(glow::BLEND);
        gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
    }
}

unsafe fn setup_tex_params(gl: &glow::Context) {
    gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
    gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
    gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
    gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
}

unsafe fn compile_shader(gl: &glow::Context, shader_type: u32, source: &str) -> Result<glow::Shader, String> {
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

unsafe fn link_program(gl: &glow::Context, vs_src: &str, fs_src: &str) -> Result<glow::Program, String> {
    let vs = compile_shader(gl, glow::VERTEX_SHADER, vs_src)?;
    let fs = compile_shader(gl, glow::FRAGMENT_SHADER, fs_src)?;
    let prog = gl.create_program()?;
    gl.attach_shader(prog, vs);
    gl.attach_shader(prog, fs);
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

unsafe fn compile_downsample(gl: &glow::Context) -> Result<DownsampleProgram, String> {
    let prog = link_program(gl, VERTEX_SHADER_SRC, DOWNSAMPLE_SHADER_SRC)?;
    let u_texture = gl.get_uniform_location(prog, "u_texture").ok_or("u_texture not found")?;
    let u_halfpixel = gl.get_uniform_location(prog, "u_halfpixel").ok_or("u_halfpixel not found")?;
    let u_offset = gl.get_uniform_location(prog, "u_offset").ok_or("u_offset not found")?;
    let a_position = gl.get_attrib_location(prog, "a_position").ok_or("a_position not found")?;

    Ok(DownsampleProgram {
        program: prog,
        u_texture,
        u_halfpixel,
        u_offset,
        a_position,
    })
}

unsafe fn compile_upsample(gl: &glow::Context) -> Result<UpsampleProgram, String> {
    let prog = link_program(gl, VERTEX_SHADER_SRC, UPSAMPLE_SHADER_SRC)?;
    let u_texture = gl.get_uniform_location(prog, "u_texture").ok_or("u_texture not found")?;
    let u_halfpixel = gl.get_uniform_location(prog, "u_halfpixel").ok_or("u_halfpixel not found")?;
    let u_offset = gl.get_uniform_location(prog, "u_offset").ok_or("u_offset not found")?;
    let a_position = gl.get_attrib_location(prog, "a_position").ok_or("a_position not found")?;

    Ok(UpsampleProgram {
        program: prog,
        u_texture,
        u_halfpixel,
        u_offset,
        a_position,
    })
}

unsafe fn compile_composite(gl: &glow::Context) -> Result<CompositeProgram, String> {
    let prog = link_program(gl, VERTEX_SHADER_SRC, COMPOSITE_SHADER_SRC)?;
    let u_texture = gl.get_uniform_location(prog, "u_texture").ok_or("u_texture not found")?;
    let u_rect_size = gl.get_uniform_location(prog, "u_rect_size").ok_or("u_rect_size not found")?;
    let u_radius = gl.get_uniform_location(prog, "u_radius").ok_or("u_radius not found")?;
    let u_opacity = gl.get_uniform_location(prog, "u_opacity").ok_or("u_opacity not found")?;
    let u_tint = gl.get_uniform_location(prog, "u_tint").ok_or("u_tint not found")?;
    let u_noise = gl.get_uniform_location(prog, "u_noise").ok_or("u_noise not found")?;
    let u_brightness = gl.get_uniform_location(prog, "u_brightness").ok_or("u_brightness not found")?;
    let u_contrast = gl.get_uniform_location(prog, "u_contrast").ok_or("u_contrast not found")?;
    let a_position = gl.get_attrib_location(prog, "a_position").ok_or("a_position not found")?;

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
        a_position,
    })
}
