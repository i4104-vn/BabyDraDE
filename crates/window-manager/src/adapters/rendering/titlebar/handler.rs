//! Compositor-owned titlebar rendering and hit testing.

use cairo::{Context, Format, ImageSurface};
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            element::{
                memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
                Kind,
            },
            glow::GlowRenderer,
        },
    },
    desktop::Window,
    utils::{Logical, Point, Scale, Transform},
};

use crate::adapters::rendering::elements::types::CustomRenderElements;
use crate::domain::geometry::Point as PurePoint;
use crate::domain::TitlebarLayout;

pub(crate) use common::constants::{CORNER_RADIUS, TITLEBAR_BUTTON_RADIUS, TITLEBAR_HEIGHT};
use common::Action;

/// Converts a Smithay location to the pure point expected by the layout rule.
fn to_pure_location(location: Point<i32, Logical>) -> PurePoint<i32> {
    PurePoint::from((location.x, location.y))
}
pub(crate) fn render(
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
            || pointer.y >= location.y as f64 + TITLEBAR_HEIGHT as f64)
    {
        return None;
    }
    let width = (window.geometry().size.w as f64 * scale.x).round().max(1.0) as i32;
    let height = (TITLEBAR_HEIGHT as f64 * scale.y).round().max(1.0) as i32;
    let mut surface = ImageSurface::create(Format::ARgb32, width, height).ok()?;
    {
        let ctx = Context::new(&surface).ok()?;
        ctx.scale(scale.x, scale.y);
        let w_local = width as f64 / scale.x;

        let color = super::color::current_color(window);

        // Clear surface to ensure transparency outside rounded corners
        ctx.set_operator(cairo::Operator::Clear);
        ctx.paint().ok()?;
        ctx.set_operator(cairo::Operator::Over);

        // Paint the background matching the app's sampled color with frosted-glass translucency
        let h_local = TITLEBAR_HEIGHT as f64;
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

        let layout = TitlebarLayout::new(w_local, maximized);
        let cy = layout.button(Action::Close).y;
        let close_x = layout.button(Action::Close).x;
        let max_x = layout.button(Action::Maximize).x;
        let min_x = layout.button(Action::Minimize).x;

        let frame_y = layout.frame_y(to_pure_location(location), maximized);
        let local_pointer =
            Point::<f64, Logical>::from((pointer.x - location.x as f64, pointer.y - frame_y));

        let in_cluster = local_pointer.x >= (min_x - TITLEBAR_BUTTON_RADIUS)
            && local_pointer.x <= (close_x + TITLEBAR_BUTTON_RADIUS)
            && local_pointer.y >= 0.0
            && local_pointer.y <= TITLEBAR_HEIGHT as f64;

        let hover_close =
            (local_pointer.x - close_x).hypot(local_pointer.y - cy) <= TITLEBAR_BUTTON_RADIUS;
        let hover_max =
            (local_pointer.x - max_x).hypot(local_pointer.y - cy) <= TITLEBAR_BUTTON_RADIUS;
        let hover_min =
            (local_pointer.x - min_x).hypot(local_pointer.y - cy) <= TITLEBAR_BUTTON_RADIUS;

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
    let titlebar_y = TitlebarLayout::new(width as f64 / scale.x, maximized)
        .frame_y(to_pure_location(location), maximized) as i32;
    let titlebar_loc = Point::<i32, Logical>::from((location.x, titlebar_y));
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
