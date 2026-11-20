//! Compositor-owned titlebar rendering and hit testing.

pub mod input;

use cairo::{Context, Format, ImageSurface};
use smithay::{
    backend::allocator::Fourcc,
    backend::renderer::{
        element::{
            memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
            Kind,
        },
        glow::GlowRenderer,
        utils::{with_renderer_surface_state},
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

/// Cached background color sampled once per window and reused every frame.
#[derive(Clone, Copy, Debug)]
pub struct WindowTitlebarColor(pub [f64; 3]);

/// Parse a CSS hex color string like `#RRGGBB` into linear `[f64; 3]`.
fn parse_hex(s: &str) -> Option<[f64; 3]> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some([r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0])
}

/// Read the first `background #RRGGBB` line from a kitty conf/theme file.
fn read_kitty_config() -> Option<[f64; 3]> {
    let home = std::env::var("HOME").ok()?;
    // theme.conf overrides kitty.conf for color settings
    for name in ["theme.conf", "kitty.conf"] {
        let path = format!("{home}/.config/kitty/{name}");
        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('#') {
                    continue; // comment
                }
                let lower = trimmed.to_lowercase();
                if lower.starts_with("background") {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Some(c) = parse_hex(parts[1]) {
                            return Some(c);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Read `background.color` from alacritty config (TOML).
fn read_alacritty_config() -> Option<[f64; 3]> {
    let home = std::env::var("HOME").ok()?;
    for name in ["alacritty/alacritty.toml", "alacritty.toml", "alacritty/alacritty.yml"] {
        let path = format!("{home}/.config/{name}");
        if let Ok(content) = std::fs::read_to_string(&path) {
            // TOML: colors.primary.background = "#181a20"
            // YAML: background: '0x181a20'
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("background") {
                    // TOML: background = "#rrggbb"  or  background = '0xrrggbb'
                    if let Some(pos) = trimmed.find('=') {
                        let val = trimmed[pos + 1..].trim().trim_matches(|c| c == '"' || c == '\'');
                        let hex = if val.starts_with("0x") || val.starts_with("0X") {
                            &val[2..]
                        } else {
                            val.trim_start_matches('#')
                        };
                        if hex.len() == 6 {
                            if let Some(c) = parse_hex(hex) {
                                return Some(c);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Get the app_id string of the window, lowercased.
fn window_app_id(window: &Window) -> Option<String> {
    let toplevel = window.toplevel()?;
    with_states(toplevel.wl_surface(), |states| {
        states
            .data_map
            .get::<XdgToplevelSurfaceData>()
            .and_then(|d| d.lock().ok().and_then(|r| r.app_id.clone()))
    })
    .map(|s| s.to_lowercase())
}

/// Attempt to read the actual background color from the app's own config file.
fn color_from_app_config(window: &Window) -> Option<[f64; 3]> {
    let app_id = window_app_id(window)?;
    if app_id.contains("kitty") {
        return read_kitty_config();
    }
    if app_id.contains("alacritty") {
        return read_alacritty_config();
    }
    None
}

/// Hardcoded per-app sensible dark fallbacks used when config files cannot be read.
fn app_fallback_color(window: &Window) -> [f64; 3] {
    let app_id = window_app_id(window).unwrap_or_default();
    if app_id.contains("kitty") {
        // BabyDra / Material Dark: #14141c
        [0.078, 0.078, 0.110]
    } else if app_id.contains("alacritty") || app_id.contains("foot") || app_id.contains("wezterm") || app_id.contains("terminal") {
        // Generic dark terminal: #181820
        [0.094, 0.094, 0.125]
    } else if app_id.contains("code") {
        // VS Code: #1e1e1e
        [0.118, 0.118, 0.118]
    } else {
        // Neutral dark: #12121a
        [0.071, 0.071, 0.102]
    }
}

/// Read one ARGB8888/XRGB8888 pixel from an SHM-backed Wayland surface.
/// Returns `None` for GPU (dmabuf/EGL) clients which never use SHM.
fn sample_shm_color(window: &Window) -> Option<[f64; 3]> {
    let toplevel = window.toplevel()?;
    with_states(toplevel.wl_surface(), |states| {
        let ud = states
            .data_map
            .get::<smithay::backend::renderer::utils::RendererSurfaceStateUserData>()?;
        let state = ud.lock().ok()?;
        let buffer = state.buffer()?;
        smithay::wayland::shm::with_buffer_contents(buffer, |ptr, len, data| {
            if data.width <= 0 || data.height <= 0 || len < 4 {
                return None;
            }
            // Sample 5 inset points and pick the most-common (background) color
            let w = data.width;
            let h = data.height;
            let stride = data.stride as usize;
            // Inset 8px from edges to skip window borders/shadows
            let candidates = [
                (8.min(w / 4), 8.min(h / 4)),
                (w / 2, 8.min(h / 4)),
                (w - 8.min(w / 4), 8.min(h / 4)),
                (w / 2, h / 2),
                (8.min(w / 4), h / 2),
            ];
            let mut colors: Vec<[f64; 3]> = candidates
                .iter()
                .filter_map(|&(sx, sy)| {
                    let off = sy as usize * stride + sx as usize * 4;
                    if off + 4 > len {
                        return None;
                    }
                    let s = unsafe { std::slice::from_raw_parts(ptr.add(off), 4) };
                    // ARGB8888 little-endian: B G R A
                    let a = s[3];
                    if a < 10 {
                        return None;
                    }
                    Some([s[2] as f64 / 255.0, s[1] as f64 / 255.0, s[0] as f64 / 255.0])
                })
                .collect();
            if colors.is_empty() {
                return None;
            }
            // Return the darkest sample — most likely the background, not text
            colors.sort_by(|a, b| {
                let la = 0.299 * a[0] + 0.587 * a[1] + 0.114 * a[2];
                let lb = 0.299 * b[0] + 0.587 * b[1] + 0.114 * b[2];
                la.partial_cmp(&lb).unwrap_or(std::cmp::Ordering::Equal)
            });
            Some(colors[0])
        })
        .ok()?
    })
}

/// Resolve the best-match background color for a window.
/// Priority: (1) app config file → (2) SHM pixel readback → (3) hardcoded app fallback.
fn resolve_color(renderer: &mut GlowRenderer, window: &Window) -> [f64; 3] {
    // Priority 1: read the app's own config (most accurate, works for GPU clients)
    if let Some(c) = color_from_app_config(window) {
        return c;
    }

    // Priority 2: direct SHM buffer pixel (works for CPU-rendered clients)
    if let Some(c) = sample_shm_color(window) {
        // Validate: reject excessively bright samples (likely hit a text pixel)
        let lum = 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2];
        if lum < 0.75 {
            return c;
        }
    }

    // Priority 3: try GL texture read for GPU clients that expose TEXTURE_2D
    if let Some(c) = sample_gl_color(renderer, window) {
        return c;
    }

    // Priority 4: hardcoded per-app fallback
    app_fallback_color(window)
}

/// Best-effort OpenGL pixel readback from a standard TEXTURE_2D surface.
/// Skips silently for external OES textures (dmabuf GPU clients — handled by config reading).
fn sample_gl_color(renderer: &mut GlowRenderer, window: &Window) -> Option<[f64; 3]> {
    use glow::HasContext;

    let toplevel = window.toplevel()?;
    let surface = toplevel.wl_surface();
    let context_id = renderer.context_id();

    let (tex_id, size, y_inverted) = with_renderer_surface_state(surface, |state| {
        let tex = state.texture::<smithay::backend::renderer::gles::GlesTexture>(context_id)?;
        Some((tex.tex_id(), tex.size(), tex.is_y_inverted()))
    })??;

    if size.w <= 0 || size.h <= 0 {
        return None;
    }

    let glow_tex = std::num::NonZeroU32::new(tex_id).map(glow::NativeTexture)?;

    // Compute UV for 8px inset from the top of the client area
    let h = size.h as f32;
    let uv_y: f32 = if y_inverted {
        (8.0 / h).clamp(0.01, 0.5)
    } else {
        (1.0 - 8.0 / h).clamp(0.5, 0.99)
    };
    let sample_uv = (0.5f32, uv_y);

    renderer
        .with_context(|gl| unsafe {
            // Try attaching as plain TEXTURE_2D — fast path, works for SHM-imported textures
            let saved_fbo = gl.get_parameter_i32(glow::FRAMEBUFFER_BINDING).max(0) as u32;
            let saved_fbo = std::num::NonZeroU32::new(saved_fbo).map(glow::NativeFramebuffer);

            let fbo = gl.create_framebuffer().ok()?;
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(glow_tex),
                0,
            );
            let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
            if status != glow::FRAMEBUFFER_COMPLETE {
                // dmabuf external texture — cannot read this way, return None
                gl.bind_framebuffer(glow::FRAMEBUFFER, saved_fbo);
                gl.delete_framebuffer(fbo);
                return None;
            }

            // Pixel coordinate from UV
            let px = (sample_uv.0 * size.w as f32) as i32;
            let py = (sample_uv.1 * size.h as f32) as i32;
            let mut pixel = [0u8; 4];
            gl.read_pixels(
                px.clamp(0, size.w - 1),
                py.clamp(0, size.h - 1),
                1, 1,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(Some(&mut pixel)),
            );
            gl.bind_framebuffer(glow::FRAMEBUFFER, saved_fbo);
            gl.delete_framebuffer(fbo);

            if pixel[3] < 10 {
                return None;
            }
            Some([pixel[0] as f64 / 255.0, pixel[1] as f64 / 255.0, pixel[2] as f64 / 255.0])
        })
        .ok()?
}

/// Get or lazily compute the titlebar background color for this window.
///
/// Cache strategy:
/// - If the color was resolved from the app's own **config file** (most accurate path),
///   store it permanently — config files don't change while the app is running.
/// - Otherwise (SHM readback / GL readback / hardcoded fallback), do NOT cache yet;
///   retry every frame until the app_id is available and we can read the config.
///   This prevents "poison cache" on first frame when `app_id` is still `None`.
pub fn get_or_sample_color(renderer: &mut GlowRenderer, window: &Window) -> [f64; 3] {
    // Return permanently cached value (only stored after a config-file hit)
    if let Some(cached) = window.user_data().get::<WindowTitlebarColor>() {
        return cached.0;
    }

    // Try config file first — most reliable, works for GPU/dmabuf clients
    if let Some(color) = color_from_app_config(window) {
        // Got a definitive answer from the app's own config: cache it forever
        window
            .user_data()
            .insert_if_missing(|| WindowTitlebarColor(color));
        return color;
    }

    // app_id not yet available or app not recognised — fall through to runtime sampling.
    // Do NOT cache the result so we retry every frame until app_id is set.
    resolve_color(renderer, window)
}

// Keep old public name as an alias for external callers / tests
pub use get_or_sample_color as sample_window_color_cached;

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
    _opacity: f32,
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
            // Semi-transparent tint over the BlurRenderElement beneath.
            // The blur backdrop already renders the frosted-glass effect; this layer
            // adds the app's tint color at ~75% opacity so the blur "shows through"
            // while still matching the app's overall dark tone.
            ctx.set_source_rgba(color[0], color[1], color[2], 0.75);
        } else {
            // No blur — fully opaque solid color matching the app background exactly.
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
