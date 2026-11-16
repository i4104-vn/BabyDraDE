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
    let y = if maximized {
        location.y as f64
    } else {
        location.y as f64 - HEIGHT as f64
    };
    if point.x < location.x as f64
        || point.x >= location.x as f64 + width
        || point.y < y
        || point.y >= location.y as f64
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
    // A maximized window can start below a desktop panel/topbar. Reveal the
    // frame relative to the window's own top edge, not the output's y=0.
    if maximized && (pointer.y < location.y as f64 || pointer.y > location.y as f64 + 8.0) {
        return None;
    }
    let width = (window.geometry().size.w as f64 * scale.x).round().max(1.0) as i32;
    let height = (HEIGHT as f64 * scale.y).round().max(1.0) as i32;
    let mut surface = ImageSurface::create(Format::ARgb32, width, height).ok()?;
    {
        let ctx = Context::new(&surface).ok()?;
        ctx.scale(scale.x, scale.y);
        ctx.set_source_rgba(0.10, 0.11, 0.14, 0.98);
        ctx.rectangle(0.0, 0.0, width as f64 / scale.x, HEIGHT as f64);
        ctx.fill().ok()?;
        ctx.set_source_rgba(1.0, 1.0, 1.0, 0.08);
        ctx.rectangle(0.0, HEIGHT as f64 - 1.0, width as f64 / scale.x, 1.0);
        ctx.fill().ok()?;
        for (x, color) in [
            (width as f64 / scale.x - 22.0, (0.95, 0.32, 0.30)),
            (width as f64 / scale.x - 46.0, (0.98, 0.72, 0.25)),
            (width as f64 / scale.x - 70.0, (0.30, 0.82, 0.45)),
        ] {
            ctx.set_source_rgb(color.0, color.1, color.2);
            ctx.arc(x, HEIGHT as f64 / 2.0, 6.0, 0.0, std::f64::consts::TAU);
            ctx.fill().ok()?;
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
