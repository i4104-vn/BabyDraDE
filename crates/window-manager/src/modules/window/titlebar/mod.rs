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

use crate::modules::render::elements::CustomRenderElements;
use crate::shared::types::TitlebarLayout;

pub(crate) use common::constants::TITLEBAR_HEIGHT;

/// Renders the compositor-owned titlebar for a single frame.
///
/// # Arguments
///
/// * `renderer` - Glow renderer that receives the finished element.
/// * `window` - Window whose titlebar is drawn.
/// * `location` - Window location in logical coordinates.
/// * `scale` - Output scale used to size the backing surface.
/// * `pointer` - Pointer position in global logical coordinates.
/// * `should_blur` - Whether the background should be translucent.
/// * `opacity` - Translucency level when blur is enabled.
///
/// # Returns
///
/// A memory render element, or `None` when the titlebar is hidden or drawing fails.
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
        let style = paint::PaintStyle {
            color: theme::current_color(window),
            should_blur,
            opacity,
            maximized,
        };
        paint::draw_background(&ctx, w_local, &style)?;

        let layout = TitlebarLayout::new(w_local, maximized);
        let controls = paint::Controls::new(&layout, location, pointer, maximized);
        paint::draw_buttons(&ctx, &controls, maximized)?;
    }
    surface.flush();

    upload(
        renderer,
        &mut surface,
        width,
        height,
        location,
        scale,
        maximized,
    )
}

/// Copies the painted surface into a memory render element.
///
/// # Arguments
///
/// * `renderer` - Glow renderer that receives the finished element.
/// * `surface` - Fully painted titlebar surface.
/// * `width` - Surface width in physical pixels.
/// * `height` - Surface height in physical pixels.
/// * `location` - Window location in logical coordinates.
/// * `scale` - Output scale used for placement.
/// * `maximized` - Whether the window is maximized.
///
/// # Returns
///
/// The titlebar render element, or `None` when the buffer cannot be uploaded.
fn upload(
    renderer: &mut GlowRenderer,
    surface: &mut ImageSurface,
    width: i32,
    height: i32,
    location: Point<i32, Logical>,
    scale: Scale<f64>,
    maximized: bool,
) -> Option<CustomRenderElements> {
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
        .frame_y(paint::to_pure_location(location), maximized) as i32;
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

pub(crate) mod paint;

pub(crate) mod theme;
