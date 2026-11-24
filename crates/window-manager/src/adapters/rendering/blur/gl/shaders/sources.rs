//! GLSL shader sources for the blur pipeline.

pub(crate) const QUAD_VERTICES: [f32; 8] = [-1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0];

pub(super) const VERTEX_SHADER_SRC: &str = r#"
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

pub(super) const DOWNSAMPLE_SHADER_SRC: &str = r#"
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

pub(super) const UPSAMPLE_SHADER_SRC: &str = r#"
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

pub(super) const COMPOSITE_SHADER_SRC: &str = r#"
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

// High quality triangular dithering function that eliminates color banding without coarse grain
float triangular_dither(vec2 p) {
    vec2 p1 = fract(p * vec2(443.897, 441.423));
    p1 += dot(p1, p1.yx + 19.19);
    float r1 = fract((p1.x + p1.y) * p1.x);

    vec2 p2 = fract(p * vec2(397.113, 467.779));
    p2 += dot(p2, p2.yx + 31.37);
    float r2 = fract((p2.x + p2.y) * p2.x);

    return r1 + r2 - 1.0;
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

    float n = triangular_dither(v_norm_coord * u_rect_size) * u_noise * 0.5;
    color.rgb += vec3(n);

    gl_FragColor = vec4(color.rgb * alpha, alpha);
}
"#;
