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
};

use super::elements::types::CustomRenderElements;

pub const HEIGHT: i32 = 34;
const BUTTON: f64 = 12.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Close,
    Minimize,
    Maximize,
    Move,
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
            let r = 20.0f64;
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
        let h_local = HEIGHT as f64;

        ctx.set_source_rgba(0.10, 0.11, 0.14, 0.98);
        if maximized {
            ctx.rectangle(0.0, 0.0, w_local, h_local);
        } else {
            let r = 20.0f64.min(w_local / 2.0).min(h_local);
            ctx.new_sub_path();
            ctx.arc(w_local - r, r, r, -std::f64::consts::FRAC_PI_2, 0.0);
            ctx.line_to(w_local, h_local);
            ctx.line_to(0.0, h_local);
            ctx.arc(r, r, r, std::f64::consts::PI, 3.0 * std::f64::consts::FRAC_PI_2);
            ctx.close_path();
        }
        ctx.fill().ok()?;

        ctx.set_source_rgba(1.0, 1.0, 1.0, 0.08);
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
        let local_pointer = Point::<f64, Logical>::from((
            pointer.x - location.x as f64,
            pointer.y - frame_y,
        ));

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
