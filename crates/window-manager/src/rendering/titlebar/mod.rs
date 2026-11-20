//! Compositor-owned titlebar rendering and hit testing.

pub mod input;

use cairo::{Context, Format, ImageSurface};
use glow::HasContext;
use smithay::{
    backend::allocator::Fourcc,
    backend::renderer::{
        element::{
            memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
            Kind,
        },
        glow::GlowRenderer,
        utils::{import_surface_tree, with_renderer_surface_state},
        Renderer, Texture,
    },
    desktop::Window,
    utils::{Logical, Point, Scale, Transform},
    wayland::{compositor::with_states, shell::xdg::XdgToplevelSurfaceData},
};

use super::elements::types::CustomRenderElements;

pub use crate::enums::constants::{
    CORNER_RADIUS, TITLEBAR_BUTTON_RADIUS as BUTTON, TITLEBAR_HEIGHT as HEIGHT,
};
pub use crate::enums::Action;

/// Cached background color extracted from the client window surface.
#[derive(Clone, Copy, Debug)]
pub struct WindowTitlebarColor(pub [f64; 3]);

/// Constant for the external OpenGL ES texture target.
const GL_TEXTURE_EXTERNAL_OES: u32 = 0x8D65;

/// Derives an intelligent app-aware fallback background color from the window's app_id.
pub fn app_fallback_color(window: &Window) -> [f64; 3] {
    let toplevel = window.toplevel();
    let app_id = toplevel.and_then(|t| {
        with_states(t.wl_surface(), |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .and_then(|d| d.lock().ok().and_then(|role| role.app_id.clone()))
        })
    });

    let app_lower = app_id.as_deref().unwrap_or("").to_lowercase();
    if app_lower.contains("kitty") {
        // Kitty Material / BabyDra Dark background (#14141c)
        [0.078, 0.078, 0.110]
    } else if app_lower.contains("alacritty")
        || app_lower.contains("foot")
        || app_lower.contains("wezterm")
        || app_lower.contains("terminal")
    {
        // Dark terminal background (#181825 / #1e1e2e)
        [0.094, 0.094, 0.145]
    } else if app_lower.contains("code") {
        // VS Code dark background (#1e1e1e)
        [0.118, 0.118, 0.118]
    } else if app_lower.contains("nautilus") || app_lower.contains("thunar") {
        // Modern dark file manager (#242424)
        [0.141, 0.141, 0.141]
    } else {
        // Neutral modern dark base (#14141c)
        [0.078, 0.078, 0.110]
    }
}

/// Attempts to sample background color directly from an SHM buffer.
fn sample_shm_color(window: &Window) -> Option<[f64; 3]> {
    let toplevel = window.toplevel()?;
    let surface = toplevel.wl_surface();
    with_states(surface, |states| {
        let state_user_data = states
            .data_map
            .get::<smithay::backend::renderer::utils::RendererSurfaceStateUserData>()?;
        let state = state_user_data.lock().ok()?;
        let buffer = state.buffer()?;
        smithay::wayland::shm::with_buffer_contents(buffer, |ptr, len, data| {
            if data.width <= 0 || data.height <= 0 || len == 0 {
                return None;
            }
            let sample_x = 6.min(data.width - 1).max(0);
            let sample_y = 6.min(data.height - 1).max(0);
            let stride = data.stride as usize;
            let offset = sample_y as usize * stride + sample_x as usize * 4;
            if offset + 4 <= len {
                let slice = unsafe { std::slice::from_raw_parts(ptr.add(offset), 4) };
                // Format ARGB8888 / XRGB8888 is BGRA in little-endian memory
                let b = slice[0] as f64 / 255.0;
                let g = slice[1] as f64 / 255.0;
                let r = slice[2] as f64 / 255.0;
                let a = slice[3] as f64 / 255.0;
                if a > 0.05 {
                    Some([r, g, b])
                } else {
                    None
                }
            } else {
                None
            }
        })
        .ok()?
    })
}

/// Samples the color of an OpenGL client texture using a 1x1 renderbuffer pass.
/// Supports both standard 2D textures and external OES dmabuf textures.
unsafe fn sample_gl_texture(
    gl: &glow::Context,
    glow_tex: glow::NativeTexture,
    sample_points: &[(f32, f32)],
) -> Option<[f64; 3]> {
    let saved_fbo = gl.get_parameter_i32(glow::FRAMEBUFFER_BINDING).max(0) as u32;
    let saved_fbo = std::num::NonZeroU32::new(saved_fbo).map(glow::NativeFramebuffer);

    let saved_prog = gl.get_parameter_i32(glow::CURRENT_PROGRAM).max(0) as u32;
    let saved_prog = std::num::NonZeroU32::new(saved_prog).map(glow::NativeProgram);

    let mut saved_viewport = [0i32; 4];
    gl.get_parameter_i32_slice(glow::VIEWPORT, &mut saved_viewport);

    let saved_tex = gl.get_parameter_i32(glow::TEXTURE_BINDING_2D).max(0) as u32;
    let saved_tex = std::num::NonZeroU32::new(saved_tex).map(glow::NativeTexture);

    let saved_vbo = gl.get_parameter_i32(glow::ARRAY_BUFFER_BINDING).max(0) as u32;
    let saved_vbo = std::num::NonZeroU32::new(saved_vbo).map(glow::NativeBuffer);

    // Create 1x1 FBO with Renderbuffer storage (guaranteed complete in GLES 2/3)
    let fbo = match gl.create_framebuffer() {
        Ok(fbo) => fbo,
        Err(_) => return None,
    };
    let rbo = match gl.create_renderbuffer() {
        Ok(rbo) => rbo,
        Err(_) => {
            gl.delete_framebuffer(fbo);
            return None;
        }
    };

    gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rbo));
    gl.renderbuffer_storage(glow::RENDERBUFFER, glow::RGBA8, 1, 1);
    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
    gl.framebuffer_renderbuffer(
        glow::FRAMEBUFFER,
        glow::COLOR_ATTACHMENT0,
        glow::RENDERBUFFER,
        Some(rbo),
    );

    if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
        gl.bind_framebuffer(glow::FRAMEBUFFER, saved_fbo);
        gl.delete_framebuffer(fbo);
        gl.delete_renderbuffer(rbo);
        return None;
    }

    // Clear prior errors
    while gl.get_error() != glow::NO_ERROR {}

    // Check if texture target is TEXTURE_2D or TEXTURE_EXTERNAL_OES
    gl.bind_texture(glow::TEXTURE_2D, Some(glow_tex));
    let is_external = gl.get_error() != glow::NO_ERROR;
    let target = if is_external {
        gl.bind_texture(GL_TEXTURE_EXTERNAL_OES, Some(glow_tex));
        GL_TEXTURE_EXTERNAL_OES
    } else {
        glow::TEXTURE_2D
    };

    let vs_src = "precision mediump float;\nattribute vec2 a_pos;\nvarying vec2 v_uv;\nuniform vec2 u_sample_uv;\nvoid main() {\n    v_uv = u_sample_uv;\n    gl_Position = vec4(a_pos, 0.0, 1.0);\n}";
    let fs_src = if is_external {
        "#extension GL_OES_EGL_image_external : require\nprecision mediump float;\nvarying vec2 v_uv;\nuniform samplerExternalOES u_tex;\nvoid main() {\n    gl_FragColor = texture2D(u_tex, v_uv);\n}"
    } else {
        "precision mediump float;\nvarying vec2 v_uv;\nuniform sampler2D u_tex;\nvoid main() {\n    gl_FragColor = texture2D(u_tex, v_uv);\n}"
    };

    let vs = match gl.create_shader(glow::VERTEX_SHADER) {
        Ok(s) => s,
        Err(_) => {
            gl.bind_framebuffer(glow::FRAMEBUFFER, saved_fbo);
            gl.delete_framebuffer(fbo);
            gl.delete_renderbuffer(rbo);
            return None;
        }
    };
    gl.shader_source(vs, vs_src);
    gl.compile_shader(vs);

    let fs = match gl.create_shader(glow::FRAGMENT_SHADER) {
        Ok(s) => s,
        Err(_) => {
            gl.delete_shader(vs);
            gl.bind_framebuffer(glow::FRAMEBUFFER, saved_fbo);
            gl.delete_framebuffer(fbo);
            gl.delete_renderbuffer(rbo);
            return None;
        }
    };
    gl.shader_source(fs, fs_src);
    gl.compile_shader(fs);

    let prog = match gl.create_program() {
        Ok(p) => p,
        Err(_) => {
            gl.delete_shader(vs);
            gl.delete_shader(fs);
            gl.bind_framebuffer(glow::FRAMEBUFFER, saved_fbo);
            gl.delete_framebuffer(fbo);
            gl.delete_renderbuffer(rbo);
            return None;
        }
    };
    gl.attach_shader(prog, vs);
    gl.attach_shader(prog, fs);
    gl.link_program(prog);

    if !gl.get_program_link_status(prog) {
        gl.delete_shader(vs);
        gl.delete_shader(fs);
        gl.delete_program(prog);
        gl.bind_framebuffer(glow::FRAMEBUFFER, saved_fbo);
        gl.delete_framebuffer(fbo);
        gl.delete_renderbuffer(rbo);
        return None;
    }

    gl.viewport(0, 0, 1, 1);
    gl.use_program(Some(prog));

    gl.active_texture(glow::TEXTURE0);
    gl.bind_texture(target, Some(glow_tex));
    if let Some(loc) = gl.get_uniform_location(prog, "u_tex") {
        gl.uniform_1_i32(Some(&loc), 0);
    }
    let uv_loc = gl.get_uniform_location(prog, "u_sample_uv");

    let quad: [f32; 8] = [-1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0];
    let vbo = gl.create_buffer().ok();
    if let Some(vbo) = vbo {
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        let bytes: &[u8] =
            core::slice::from_raw_parts(quad.as_ptr() as *const u8, quad.len() * 4);
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes, glow::STREAM_DRAW);
    }

    let pos_loc = gl.get_attrib_location(prog, "a_pos");
    if let Some(pos) = pos_loc {
        gl.enable_vertex_attrib_array(pos);
        gl.vertex_attrib_pointer_f32(pos, 2, glow::FLOAT, false, 8, 0);
    }

    let mut sampled_colors = Vec::new();
    for &(u, v) in sample_points {
        if let Some(ref loc) = uv_loc {
            gl.uniform_2_f32(Some(loc), u, v);
        }
        gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);

        let mut pixel = [0u8; 4];
        gl.read_pixels(
            0,
            0,
            1,
            1,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelPackData::Slice(Some(&mut pixel)),
        );

        if pixel[3] > 10 {
            sampled_colors.push([
                pixel[0] as f64 / 255.0,
                pixel[1] as f64 / 255.0,
                pixel[2] as f64 / 255.0,
            ]);
        }
    }

    if let Some(pos) = pos_loc {
        gl.disable_vertex_attrib_array(pos);
    }

    if let Some(vbo) = vbo {
        gl.delete_buffer(vbo);
    }
    gl.delete_shader(vs);
    gl.delete_shader(fs);
    gl.delete_program(prog);
    gl.delete_framebuffer(fbo);
    gl.delete_renderbuffer(rbo);

    if is_external {
        gl.bind_texture(GL_TEXTURE_EXTERNAL_OES, None);
    }

    // Restore saved OpenGL states
    gl.bind_framebuffer(glow::FRAMEBUFFER, saved_fbo);
    gl.bind_texture(glow::TEXTURE_2D, saved_tex);
    gl.bind_buffer(glow::ARRAY_BUFFER, saved_vbo);
    gl.use_program(saved_prog);
    gl.viewport(
        saved_viewport[0],
        saved_viewport[1],
        saved_viewport[2],
        saved_viewport[3],
    );

    if sampled_colors.is_empty() {
        return None;
    }

    if sampled_colors.len() == 1 {
        Some(sampled_colors[0])
    } else {
        // Find if any 2 colors match closely (typical of padding/background)
        for i in 0..sampled_colors.len() {
            for j in (i + 1)..sampled_colors.len() {
                let dr = (sampled_colors[i][0] - sampled_colors[j][0]).abs();
                let dg = (sampled_colors[i][1] - sampled_colors[j][1]).abs();
                let db = (sampled_colors[i][2] - sampled_colors[j][2]).abs();
                if dr + dg + db < 0.08 {
                    return Some(sampled_colors[i]);
                }
            }
        }
        // Fallback: pick the lowest luminance (dark background for dark windows)
        sampled_colors.into_iter().min_by(|a, b| {
            let lum_a = 0.299 * a[0] + 0.587 * a[1] + 0.114 * a[2];
            let lum_b = 0.299 * b[0] + 0.587 * b[1] + 0.114 * b[2];
            lum_a.partial_cmp(&lum_b).unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

/// Extracts the background color from the client window surface.
pub fn sample_window_color(renderer: &mut GlowRenderer, window: &Window) -> Option<[f64; 3]> {
    // 1. Try SHM buffer extraction (instant CPU read if client uses SHM)
    if let Some(color) = sample_shm_color(window) {
        return Some(color);
    }

    // 2. Try OpenGL texture extraction (for GPU dmabuf / EGL clients like Kitty, Alacritty)
    let toplevel = window.toplevel()?;
    let surface = toplevel.wl_surface();

    let _ = import_surface_tree(renderer, surface);

    let context_id = renderer.context_id();
    let (tex_id, size, y_inverted) = with_renderer_surface_state(surface, |state| {
        let tex = state.texture(context_id)?;
        Some((tex.tex_id(), tex.size(), tex.is_y_inverted()))
    })??;

    if size.w <= 0 || size.h <= 0 {
        return None;
    }

    let w = size.w as f32;
    let h = size.h as f32;

    let points = [
        // Point 1: top-left padding (6px, 6px)
        (
            (6.0 / w).clamp(0.01, 0.99),
            if y_inverted {
                (6.0 / h).clamp(0.01, 0.99)
            } else {
                (1.0 - 6.0 / h).clamp(0.01, 0.99)
            },
        ),
        // Point 2: top-center (w / 2, 6px)
        (
            0.5,
            if y_inverted {
                (6.0 / h).clamp(0.01, 0.99)
            } else {
                (1.0 - 6.0 / h).clamp(0.01, 0.99)
            },
        ),
        // Point 3: top-right padding (w - 12px, 6px)
        (
            (1.0 - 12.0 / w).clamp(0.01, 0.99),
            if y_inverted {
                (6.0 / h).clamp(0.01, 0.99)
            } else {
                (1.0 - 6.0 / h).clamp(0.01, 0.99)
            },
        ),
    ];

    let glow_tex = std::num::NonZeroU32::new(tex_id).map(glow::NativeTexture)?;

    renderer
        .with_context(|gl| unsafe { sample_gl_texture(gl, glow_tex, &points) })
        .ok()?
}

/// Retrieves the cached titlebar color or samples it once from the client buffer.
pub fn get_or_sample_color(renderer: &mut GlowRenderer, window: &Window) -> [f64; 3] {
    if let Some(cached) = window.user_data().get::<WindowTitlebarColor>() {
        return cached.0;
    }

    if let Some(color) = sample_window_color(renderer, window) {
        window
            .user_data()
            .insert_if_missing(|| WindowTitlebarColor(color));
        color
    } else {
        // App-aware fallback until the first non-empty frame is committed
        app_fallback_color(window)
    }
}

pub fn hit_test(
    window: &Window,
    location: Point<i32, Logical>,
    point: Point<f64, Logical>,
) -> Option<Action> {
    let state = window.toplevel()?.current_state();
    if state.decoration_mode != Some(
        smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode::ServerSide,
    ) || state.states.contains(
        smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Fullscreen,
    ) {
        return None;
    }
    let maximized = state.states.contains(
        smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Maximized,
    );
    let width = window.geometry().size.w as f64;
    hit_test_geometry(maximized, location, width, point)
}

pub fn hit_test_geometry(
    maximized: bool,
    location: Point<i32, Logical>,
    width: f64,
    point: Point<f64, Logical>,
) -> Option<Action> {
    let (y_min, y_max) = if maximized {
        (location.y as f64, location.y as f64 + HEIGHT as f64)
    } else {
        (location.y as f64 - HEIGHT as f64, location.y as f64)
    };
    if point.x < location.x as f64
        || point.x >= location.x as f64 + width
        || point.y < y_min
        || point.y >= y_max
    {
        return None;
    }
    let close = location.x as f64 + width - 22.0;
    let max = close - 24.0;
    let min = max - 24.0;
    if point.x >= close - BUTTON && point.x <= close + BUTTON {
        Some(Action::Close)
    } else if point.x >= max - BUTTON && point.x <= max + BUTTON {
        Some(Action::Maximize)
    } else if point.x >= min - BUTTON && point.x <= min + BUTTON {
        Some(Action::Minimize)
    } else {
        if !maximized {
            let r = CORNER_RADIUS;
            // Top-left curved corner
            if point.x < location.x as f64 + r && point.y < y_min + r {
                let dx = point.x - (location.x as f64 + r);
                let dy = point.y - (y_min + r);
                if dx * dx + dy * dy > r * r {
                    return None;
                }
            }
            // Top-right curved corner
            if point.x > location.x as f64 + width - r && point.y < y_min + r {
                let dx = point.x - (location.x as f64 + width - r);
                let dy = point.y - (y_min + r);
                if dx * dx + dy * dy > r * r {
                    return None;
                }
            }
        }
        Some(Action::Move)
    }
}

pub fn render(
    renderer: &mut GlowRenderer,
    window: &Window,
    location: Point<i32, Logical>,
    scale: Scale<f64>,
    pointer: Point<f64, Logical>,
    should_blur: bool,
    opacity: f32,
) -> Option<CustomRenderElements> {
    let state = window.toplevel()?.current_state();
    if state.decoration_mode != Some(
        smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode::ServerSide,
    ) || state.states.contains(
        smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Fullscreen,
    ) {
        return None;
    }
    let maximized = state.states.contains(
        smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Maximized,
    );
    let width_f = window.geometry().size.w as f64;
    // When maximized, reveal the titlebar when the pointer hovers over its top edge area
    if maximized
        && (pointer.x < location.x as f64
            || pointer.x >= location.x as f64 + width_f
            || pointer.y < location.y as f64
            || pointer.y >= location.y as f64 + HEIGHT as f64)
    {
        return None;
    }
    let width = (window.geometry().size.w as f64 * scale.x).round().max(1.0) as i32;
    let height = (HEIGHT as f64 * scale.y).round().max(1.0) as i32;
    let mut surface = ImageSurface::create(Format::ARgb32, width, height).ok()?;
    {
        let ctx = Context::new(&surface).ok()?;
        ctx.scale(scale.x, scale.y);
        let w_local = width as f64 / scale.x;

        let color = get_or_sample_color(renderer, window);

        // Clear surface to ensure transparency outside rounded corners
        ctx.set_operator(cairo::Operator::Clear);
        ctx.paint().ok()?;
        ctx.set_operator(cairo::Operator::Over);

        // Paint the background matching the app's sampled color with frosted-glass translucency
        let h_local = HEIGHT as f64;
        if !maximized {
            let r = CORNER_RADIUS.min(w_local / 2.0).min(h_local);
            ctx.new_sub_path();
            // Top-left rounded corner
            ctx.arc(r, r, r, std::f64::consts::PI, 3.0 * std::f64::consts::FRAC_PI_2);
            // Top-right rounded corner
            ctx.arc(w_local - r, r, r, -std::f64::consts::FRAC_PI_2, 0.0);
            // Bottom-right corner (straight)
            ctx.line_to(w_local, h_local);
            // Bottom-left corner (straight)
            ctx.line_to(0.0, h_local);
            ctx.close_path();
        } else {
            ctx.rectangle(0.0, 0.0, w_local, h_local);
        }

        if should_blur {
            // For blurred windows, match the window body's exact blur opacity so the titlebar
            // and app body blend seamlessly over the shared frosted glass Dual Kawase blur backdrop.
            ctx.set_source_rgba(color[0], color[1], color[2], opacity as f64);
        } else {
            // For non-blurred windows, fill with 100% solid/opaque background color matching the app.
            ctx.set_source_rgb(color[0], color[1], color[2]);
        }
        ctx.fill().ok()?;

        // Subtle 1px bottom separator adapting to dark/light background
        let luminance = 0.299 * color[0] + 0.587 * color[1] + 0.114 * color[2];
        let (sep_r, sep_g, sep_b, sep_a) = if luminance > 0.5 {
            (0.0, 0.0, 0.0, 0.12)
        } else {
            (1.0, 1.0, 1.0, 0.08)
        };
        ctx.set_source_rgba(sep_r, sep_g, sep_b, sep_a);
        ctx.rectangle(0.0, h_local - 1.0, w_local, 1.0);
        ctx.fill().ok()?;

        let cy = HEIGHT as f64 / 2.0;
        let close_x = w_local - 22.0;
        let max_x = w_local - 46.0;
        let min_x = w_local - 70.0;

        let frame_y = if maximized {
            location.y as f64
        } else {
            location.y as f64 - HEIGHT as f64
        };
        let local_pointer =
            Point::<f64, Logical>::from((pointer.x - location.x as f64, pointer.y - frame_y));

        let in_cluster = local_pointer.x >= (min_x - BUTTON)
            && local_pointer.x <= (close_x + BUTTON)
            && local_pointer.y >= 0.0
            && local_pointer.y <= HEIGHT as f64;

        let hover_close = (local_pointer.x - close_x).hypot(local_pointer.y - cy) <= BUTTON;
        let hover_max = (local_pointer.x - max_x).hypot(local_pointer.y - cy) <= BUTTON;
        let hover_min = (local_pointer.x - min_x).hypot(local_pointer.y - cy) <= BUTTON;

        // 1. Close button (Red)
        ctx.arc(close_x, cy, 6.0, 0.0, std::f64::consts::TAU);
        if hover_close {
            ctx.set_source_rgb(1.0, 0.38, 0.35);
        } else {
            ctx.set_source_rgb(0.95, 0.32, 0.30);
        }
        ctx.fill().ok()?;

        if in_cluster {
            let alpha = if hover_close { 0.95 } else { 0.65 };
            ctx.set_source_rgba(0.30, 0.0, 0.0, alpha);
            ctx.set_line_width(1.2);
            ctx.set_line_cap(cairo::LineCap::Round);

            let offset = 2.4;
            ctx.move_to(close_x - offset, cy - offset);
            ctx.line_to(close_x + offset, cy + offset);
            ctx.stroke().ok()?;

            ctx.move_to(close_x + offset, cy - offset);
            ctx.line_to(close_x - offset, cy + offset);
            ctx.stroke().ok()?;
        }

        // 2. Maximize / Restore button (Yellow)
        ctx.arc(max_x, cy, 6.0, 0.0, std::f64::consts::TAU);
        if hover_max {
            ctx.set_source_rgb(1.0, 0.78, 0.30);
        } else {
            ctx.set_source_rgb(0.98, 0.72, 0.25);
        }
        ctx.fill().ok()?;

        if in_cluster {
            let alpha = if hover_max { 0.95 } else { 0.65 };
            ctx.set_source_rgba(0.35, 0.20, 0.0, alpha);
            ctx.set_line_width(1.1);
            ctx.set_line_cap(cairo::LineCap::Square);
            ctx.set_line_join(cairo::LineJoin::Miter);

            if maximized {
                // Restore icon: two overlapping rectangles
                ctx.rectangle(max_x - 1.2, cy - 3.2, 4.4, 4.4);
                ctx.stroke().ok()?;
                ctx.rectangle(max_x - 3.2, cy - 1.2, 4.4, 4.4);
                ctx.stroke().ok()?;
            } else {
                // Maximize icon: single square
                let s = 5.0;
                ctx.rectangle(max_x - s / 2.0, cy - s / 2.0, s, s);
                ctx.stroke().ok()?;
            }
        }

        // 3. Minimize button (Green)
        ctx.arc(min_x, cy, 6.0, 0.0, std::f64::consts::TAU);
        if hover_min {
            ctx.set_source_rgb(0.35, 0.88, 0.50);
        } else {
            ctx.set_source_rgb(0.30, 0.82, 0.45);
        }
        ctx.fill().ok()?;

        if in_cluster {
            let alpha = if hover_min { 0.95 } else { 0.65 };
            ctx.set_source_rgba(0.05, 0.25, 0.10, alpha);
            ctx.set_line_width(1.2);
            ctx.set_line_cap(cairo::LineCap::Round);

            let offset = 2.8;
            ctx.move_to(min_x - offset, cy);
            ctx.line_to(min_x + offset, cy);
            ctx.stroke().ok()?;
        }
    }
    surface.flush();
    let pixels = surface.data().ok()?.to_vec();
    let mut buffer = MemoryRenderBuffer::new(
        Fourcc::Argb8888,
        (width, height),
        1,
        Transform::Normal,
        None,
    );
    buffer
        .render()
        .draw(|buf| {
            if buf.len() == pixels.len() {
                buf.copy_from_slice(&pixels);
                Ok(vec![smithay::utils::Rectangle::from_size(
                    (width, height).into(),
                )])
            } else {
                Err(())
            }
        })
        .ok()?;
    let frame_y = if maximized {
        location.y
    } else {
        location.y - HEIGHT
    };
    let point = Point::<f64, smithay::utils::Physical>::from((
        (location.x as f64 * scale.x),
        (frame_y as f64 * scale.y),
    ));
    MemoryRenderBufferRenderElement::from_buffer(
        renderer,
        point,
        &buffer,
        None,
        None,
        None,
        Kind::Unspecified,
    )
    .ok()
    .map(CustomRenderElements::Memory)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_titlebar_color_cache() {
        let user_data = smithay::utils::user_data::UserDataMap::default();
        assert!(user_data.get::<WindowTitlebarColor>().is_none());

        user_data.insert_if_missing(|| WindowTitlebarColor([0.2, 0.3, 0.4]));
        assert_eq!(
            user_data.get::<WindowTitlebarColor>().map(|c| c.0),
            Some([0.2, 0.3, 0.4])
        );

        // Subsequent insertions are ignored (sample only once)
        user_data.insert_if_missing(|| WindowTitlebarColor([0.9, 0.9, 0.9]));
        assert_eq!(
            user_data.get::<WindowTitlebarColor>().map(|c| c.0),
            Some([0.2, 0.3, 0.4])
        );
    }

    #[test]
    fn test_titlebar_hit_test_floating() {
        let loc = Point::from((100, 200));
        let width = 800.0;

        // Titlebar is at y in [200 - 34, 200) = [166, 200)
        // Buttons: close = 100 + 800 - 22 = 878
        // max = 878 - 24 = 854
        // min = 854 - 24 = 830
        assert_eq!(
            hit_test_geometry(false, loc, width, Point::from((878.0, 180.0))),
            Some(Action::Close)
        );
        assert_eq!(
            hit_test_geometry(false, loc, width, Point::from((854.0, 180.0))),
            Some(Action::Maximize)
        );
        assert_eq!(
            hit_test_geometry(false, loc, width, Point::from((830.0, 180.0))),
            Some(Action::Minimize)
        );
        assert_eq!(
            hit_test_geometry(false, loc, width, Point::from((300.0, 180.0))),
            Some(Action::Move)
        );

        // Outside titlebar
        assert_eq!(
            hit_test_geometry(false, loc, width, Point::from((300.0, 150.0))),
            None
        );
        assert_eq!(
            hit_test_geometry(false, loc, width, Point::from((300.0, 205.0))),
            None
        );
        assert_eq!(
            hit_test_geometry(false, loc, width, Point::from((50.0, 180.0))),
            None
        );

        // Rounded top-left corner should exclude points outside 20px arc
        assert_eq!(
            hit_test_geometry(false, loc, width, Point::from((102.0, 168.0))),
            None
        );
    }

    #[test]
    fn test_titlebar_hit_test_maximized() {
        let loc = Point::from((0, 32)); // under topbar
        let width = 1920.0;

        // When maximized, titlebar is at y in [32, 32 + 34) = [32, 66)
        // Buttons: close = 0 + 1920 - 22 = 1898
        // max = 1898 - 24 = 1874
        // min = 1874 - 24 = 1850
        assert_eq!(
            hit_test_geometry(true, loc, width, Point::from((1898.0, 48.0))),
            Some(Action::Close)
        );
        assert_eq!(
            hit_test_geometry(true, loc, width, Point::from((1874.0, 48.0))),
            Some(Action::Maximize)
        );
        assert_eq!(
            hit_test_geometry(true, loc, width, Point::from((1850.0, 48.0))),
            Some(Action::Minimize)
        );
        assert_eq!(
            hit_test_geometry(true, loc, width, Point::from((500.0, 48.0))),
            Some(Action::Move)
        );

        // Outside maximized titlebar
        assert_eq!(
            hit_test_geometry(true, loc, width, Point::from((500.0, 20.0))),
            None
        );
        assert_eq!(
            hit_test_geometry(true, loc, width, Point::from((500.0, 70.0))),
            None
        );
    }
}
