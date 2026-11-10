use glow::HasContext;
use smithay::utils::{Physical, Rectangle};

use super::config::BlurParams;

const QUAD_VERTICES: [f32; 8] = [-1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0];
const MAX_BLUR_PASSES: usize = 4;

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

struct DownsampleProgram {
    program: glow::Program,
    u_texture: glow::UniformLocation,
    u_halfpixel: glow::UniformLocation,
    u_offset: glow::UniformLocation,
    u_scale_uv: glow::UniformLocation,
    a_position: u32,
}

struct UpsampleProgram {
    program: glow::Program,
    u_texture: glow::UniformLocation,
    u_halfpixel: glow::UniformLocation,
    u_offset: glow::UniformLocation,
    u_scale_uv: glow::UniformLocation,
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
    u_scale_uv: glow::UniformLocation,
    a_position: u32,
}

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

struct GlStateSnapshot<'a> {
    gl: &'a glow::Context,
    framebuffer: Option<glow::NativeFramebuffer>,
    viewport: [i32; 4],
    scissor_enabled: bool,
    scissor_box: [i32; 4],
    blend_enabled: bool,
    blend_src_rgb: u32,
    blend_dst_rgb: u32,
    blend_src_alpha: u32,
    blend_dst_alpha: u32,
    blend_equation_rgb: u32,
    blend_equation_alpha: u32,
    active_texture: u32,
    active_texture_binding: Option<glow::NativeTexture>,
    texture0_binding: Option<glow::NativeTexture>,
    array_buffer: Option<glow::NativeBuffer>,
    program: Option<glow::NativeProgram>,
    vertex_attribs: Vec<(u32, bool)>,
}

impl GlStateSnapshot<'_> {
    unsafe fn capture(gl: &glow::Context, locations: [u32; 3]) -> GlStateSnapshot<'_> {
        let mut viewport = [0; 4];
        gl.get_parameter_i32_slice(glow::VIEWPORT, &mut viewport);
        let mut scissor_box = [0; 4];
        gl.get_parameter_i32_slice(glow::SCISSOR_BOX, &mut scissor_box);

        let active_texture = gl.get_parameter_i32(glow::ACTIVE_TEXTURE) as u32;
        let active_texture_binding = gl_object(
            gl.get_parameter_i32(glow::TEXTURE_BINDING_2D),
            glow::NativeTexture,
        );
        gl.active_texture(glow::TEXTURE0);
        let texture0_binding = gl_object(
            gl.get_parameter_i32(glow::TEXTURE_BINDING_2D),
            glow::NativeTexture,
        );
        gl.active_texture(active_texture);

        let mut vertex_attribs = Vec::with_capacity(locations.len());
        for location in locations {
            if vertex_attribs
                .iter()
                .any(|(existing, _)| *existing == location)
            {
                continue;
            }
            let mut enabled = [0.0];
            gl.get_vertex_attrib_parameter_f32_slice(
                location,
                glow::VERTEX_ATTRIB_ARRAY_ENABLED,
                &mut enabled,
            );
            vertex_attribs.push((location, enabled[0] != 0.0));
        }

        GlStateSnapshot {
            gl,
            framebuffer: gl_object(
                gl.get_parameter_i32(glow::FRAMEBUFFER_BINDING),
                glow::NativeFramebuffer,
            ),
            viewport,
            scissor_enabled: gl.is_enabled(glow::SCISSOR_TEST),
            scissor_box,
            blend_enabled: gl.is_enabled(glow::BLEND),
            blend_src_rgb: gl.get_parameter_i32(glow::BLEND_SRC_RGB) as u32,
            blend_dst_rgb: gl.get_parameter_i32(glow::BLEND_DST_RGB) as u32,
            blend_src_alpha: gl.get_parameter_i32(glow::BLEND_SRC_ALPHA) as u32,
            blend_dst_alpha: gl.get_parameter_i32(glow::BLEND_DST_ALPHA) as u32,
            blend_equation_rgb: gl.get_parameter_i32(glow::BLEND_EQUATION_RGB) as u32,
            blend_equation_alpha: gl.get_parameter_i32(glow::BLEND_EQUATION_ALPHA) as u32,
            active_texture,
            active_texture_binding,
            texture0_binding,
            array_buffer: gl_object(
                gl.get_parameter_i32(glow::ARRAY_BUFFER_BINDING),
                glow::NativeBuffer,
            ),
            program: gl_object(
                gl.get_parameter_i32(glow::CURRENT_PROGRAM),
                glow::NativeProgram,
            ),
            vertex_attribs,
        }
    }
}

impl Drop for GlStateSnapshot<'_> {
    fn drop(&mut self) {
        unsafe {
            let gl = self.gl;
            gl.bind_framebuffer(glow::FRAMEBUFFER, self.framebuffer);
            gl.viewport(
                self.viewport[0],
                self.viewport[1],
                self.viewport[2],
                self.viewport[3],
            );
            gl.scissor(
                self.scissor_box[0],
                self.scissor_box[1],
                self.scissor_box[2],
                self.scissor_box[3],
            );
            if self.scissor_enabled {
                gl.enable(glow::SCISSOR_TEST);
            } else {
                gl.disable(glow::SCISSOR_TEST);
            }

            gl.bind_buffer(glow::ARRAY_BUFFER, self.array_buffer);
            gl.use_program(self.program);
            gl.blend_func_separate(
                self.blend_src_rgb,
                self.blend_dst_rgb,
                self.blend_src_alpha,
                self.blend_dst_alpha,
            );
            gl.blend_equation_separate(self.blend_equation_rgb, self.blend_equation_alpha);
            if self.blend_enabled {
                gl.enable(glow::BLEND);
            } else {
                gl.disable(glow::BLEND);
            }

            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, self.texture0_binding);
            if self.active_texture != glow::TEXTURE0 {
                gl.active_texture(self.active_texture);
                gl.bind_texture(glow::TEXTURE_2D, self.active_texture_binding);
            }

            for (location, enabled) in &self.vertex_attribs {
                if *enabled {
                    gl.enable_vertex_attrib_array(*location);
                } else {
                    gl.disable_vertex_attrib_array(*location);
                }
            }
        }
    }
}

fn gl_object<T>(value: i32, create: impl FnOnce(std::num::NonZeroU32) -> T) -> Option<T> {
    std::num::NonZeroU32::new(value as u32).map(create)
}

unsafe fn setup_tex_params(gl: &glow::Context) {
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

unsafe fn compile_downsample(gl: &glow::Context) -> Result<DownsampleProgram, String> {
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

unsafe fn compile_upsample(gl: &glow::Context) -> Result<UpsampleProgram, String> {
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

unsafe fn compile_composite(gl: &glow::Context) -> Result<CompositeProgram, String> {
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

/// Projects a destination rectangle from Wayland oriented coordinates to OpenGL physical
/// framebuffer coordinates using Smithay's exact projection matrix for the current frame.
pub fn project_rect_to_gl(
    dst: Rectangle<i32, Physical>,
    projection: &[f32; 9],
    screen_w: i32,
    screen_h: i32,
) -> (i32, i32, i32, i32) {
    let x0 = dst.loc.x as f32;
    let y0 = dst.loc.y as f32;
    let x1 = x0 + dst.size.w as f32;
    let y1 = y0 + dst.size.h as f32;

    let corners = [(x0, y0), (x1, y0), (x0, y1), (x1, y1)];

    let mut min_gx = f32::INFINITY;
    let mut max_gx = f32::NEG_INFINITY;
    let mut min_gy = f32::INFINITY;
    let mut max_gy = f32::NEG_INFINITY;

    for (x, y) in corners {
        // Column-major Matrix3 * vec3(x, y, 1.0)
        let mut rx = projection[0] * x + projection[3] * y + projection[6];
        let mut ry = projection[1] * x + projection[4] * y + projection[7];
        let rw = projection[2] * x + projection[5] * y + projection[8];

        if rw.abs() > 1e-6 {
            rx /= rw;
            ry /= rw;
        }

        // OpenGL NDC [-1.0, 1.0] -> Viewport [0, screen_size]
        let gx = (rx + 1.0) * 0.5 * screen_w as f32;
        let gy = (ry + 1.0) * 0.5 * screen_h as f32;

        min_gx = min_gx.min(gx);
        max_gx = max_gx.max(gx);
        min_gy = min_gy.min(gy);
        max_gy = max_gy.max(gy);
    }

    let gl_x = min_gx.round() as i32;
    let gl_y = min_gy.round() as i32;
    let gl_w = (max_gx - min_gx).round() as i32;
    let gl_h = (max_gy - min_gy).round() as i32;

    (gl_x, gl_y, gl_w, gl_h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_rect_to_gl_normal() {
        // Identity projection for 1920x1080 display
        // M00=2/1920, M11=2/1080, M02=-1, M12=-1 (Y up)
        let mut proj = [0.0f32; 9];
        proj[0] = 2.0 / 1920.0;
        proj[4] = 2.0 / 1080.0;
        proj[6] = -1.0;
        proj[7] = -1.0;
        proj[8] = 1.0;

        let dst = Rectangle::new((100, 200).into(), (400, 300).into());
        let (x, y, w, h) = project_rect_to_gl(dst, &proj, 1920, 1080);
        assert_eq!((x, y, w, h), (100, 200, 400, 300));
    }

    #[test]
    fn test_project_rect_to_gl_rotated_90() {
        // Rotated projection for 1600x2560 hardware panel displayed at 2560x1600 (Transform::_270 render transform)
        // With gx = 1600 - ly, gy = lx
        let mut proj = [0.0f32; 9];
        // ndc_x = 1.0 - 2.0 * y / 1600.0 => proj[3] = -2.0/1600.0, proj[6] = 1.0 (row 0)
        proj[3] = -2.0 / 1600.0;
        proj[6] = 1.0;
        // ndc_y = 2.0 * x / 2560.0 - 1.0 => proj[1] = 2.0/2560.0, proj[7] = -1.0 (row 1)
        proj[1] = 2.0 / 2560.0;
        proj[7] = -1.0;
        proj[8] = 1.0;

        // Window at top-left: x=0, y=0, w=400, h=300
        let dst = Rectangle::new((0, 0).into(), (400, 300).into());
        let (x, y, w, h) = project_rect_to_gl(dst, &proj, 1600, 2560);
        // gx range = [1600 - 300, 1600] = [1300, 1600]
        // gy range = [0, 400]
        assert_eq!((x, y, w, h), (1300, 0, 300, 400));
    }
}
