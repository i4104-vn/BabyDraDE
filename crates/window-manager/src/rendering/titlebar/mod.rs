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
    },
    desktop::Window,
    utils::{Logical, Point, Scale, Transform},
    wayland::compositor::with_states,
};

use super::elements::types::CustomRenderElements;

pub use crate::enums::constants::{
    CORNER_RADIUS, TITLEBAR_BUTTON_RADIUS as BUTTON, TITLEBAR_HEIGHT as HEIGHT,
};
pub use crate::enums::Action;

/// Cached background color sampled once per window and reused every frame.
#[derive(Clone, Copy, Debug)]
pub struct WindowTitlebarColor(pub [f64; 3]);

/// Return the representative color from pixels read from the client surface.
///
/// Grouping near-identical pixels prevents a glyph, icon, or antialiased edge
/// from becoming the titlebar color.  Unlike choosing the darkest pixel, this
/// also preserves light application themes.
fn pick_dominant_color(samples: &[[f64; 3]]) -> Option<[f64; 3]> {
    samples.iter().copied().max_by_key(|candidate| {
        samples
            .iter()
            .filter(|sample| {
                let dr = candidate[0] - sample[0];
                let dg = candidate[1] - sample[1];
                let db = candidate[2] - sample[2];
                dr * dr + dg * dg + db * db < 0.008
            })
            .count()
    })
}

/// Read representative pixels from a GPU-backed client buffer.
fn sample_dmabuf_color(window: &Window) -> Option<[f64; 3]> {
    let toplevel = window.toplevel()?;
    with_states(toplevel.wl_surface(), |states| {
        let surface_state = states
            .data_map
            .get::<smithay::backend::renderer::utils::RendererSurfaceStateUserData>(
        )?;
        let state = surface_state.lock().ok()?;
        let buffer = state.buffer()?;
        let dmabuf = smithay::wayland::dmabuf::get_dmabuf(buffer).ok()?;

        use smithay::backend::allocator::Buffer as _;
        let size = dmabuf.size();
        if size.w <= 0 || size.h <= 0 {
            return None;
        }
        let stride = dmabuf.strides().next()? as usize;
        let format = dmabuf.format();
        let mapping = dmabuf
            .map_plane(
                0,
                smithay::backend::allocator::dmabuf::DmabufMappingMode::READ,
            )
            .ok()?;
        let pixels =
            unsafe { std::slice::from_raw_parts(mapping.ptr() as *const u8, mapping.length()) };
        let samples = sample_pixels(pixels, size.w, size.h, stride, format.code);
        pick_dominant_color(&samples)
    })
}

fn sample_pixels(
    pixels: &[u8],
    width: i32,
    height: i32,
    stride: usize,
    format: Fourcc,
) -> Vec<[f64; 3]> {
    let points = [
        (8.min(width / 4), 8.min(height / 4)),
        (width / 2, 8.min(height / 4)),
        (width - 1 - 8.min(width / 4), 8.min(height / 4)),
        (width / 2, height / 2),
        (8.min(width / 4), height / 2),
    ];
    points
        .iter()
        .filter_map(|&(x, y)| {
            let offset = y as usize * stride + x as usize * 4;
            let pixel = pixels.get(offset..offset + 4)?;
            let rgb = match format {
                Fourcc::Argb8888 | Fourcc::Xrgb8888 => [pixel[2], pixel[1], pixel[0]],
                Fourcc::Abgr8888 | Fourcc::Xbgr8888 => [pixel[0], pixel[1], pixel[2]],
                _ => return None,
            };
            Some(rgb.map(|channel| channel as f64 / 255.0))
        })
        .collect()
}

/// Read representative pixels from a CPU-backed client buffer.
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
            let pixels = unsafe { std::slice::from_raw_parts(ptr, len) };
            let format = smithay::wayland::shm::shm_format_to_fourcc(data.format)?;
            pick_dominant_color(&sample_pixels(
                pixels,
                data.width,
                data.height,
                data.stride as usize,
                format,
            ))
        })
        .ok()?
    })
}

/// Resolve the titlebar color only from the contents actually rendered by the
/// application. GPU clients are sampled first; SHM clients use the same logic.
fn resolve_color(window: &Window) -> Option<[f64; 3]> {
    sample_dmabuf_color(window).or_else(|| sample_shm_color(window))
}

/// Get or lazily compute (once) the titlebar background color for this window.
///
/// Automatically caches the resolved color in the window's user data so that
/// resolution is performed exactly once per window lifetime. A fallback is not
/// cached, so the first committed client buffer always replaces it.
pub fn get_or_sample_color(window: &Window) -> [f64; 3] {
    if let Some(cached) = window.user_data().get::<WindowTitlebarColor>() {
        return cached.0;
    }

    if let Some(color) = resolve_color(window) {
        window
            .user_data()
            .insert_if_missing(|| WindowTitlebarColor(color));
        return color;
    }

    // There is no application buffer yet. This is intentionally temporary and
    // never replaces a color sampled from the application.
    [0.0, 0.0, 0.0]
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

        let color = get_or_sample_color(window);

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
            ctx.arc(
                r,
                r,
                r,
                std::f64::consts::PI,
                3.0 * std::f64::consts::FRAC_PI_2,
            );
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
            ctx.set_source_rgba(color[0], color[1], color[2], opacity as f64);
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
    let titlebar_loc = Point::<i32, Logical>::from((location.x, frame_y));
    let titlebar_phys: Point<i32, smithay::utils::Physical> =
        titlebar_loc.to_physical_precise_round(scale);
    let point = titlebar_phys.to_f64();
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
