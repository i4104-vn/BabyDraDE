//! Shared off-screen Cairo buffer helpers.

use cairo::{Context, Format, ImageSurface};
use smithay::utils::Scale;

/// Renders `draw_fn` into an ARgb32 buffer of `width` x `height`.
///
/// The Cairo context is scaled by `scale` and translated by `origin`, so the
/// drawing code can keep using logical screen coordinates while only the
/// visible region is allocated.
pub(crate) fn render_pixels<F>(
    width: i32,
    height: i32,
    scale: Scale<f64>,
    origin: (i32, i32),
    draw_fn: F,
) -> Option<Vec<u8>>
where
    F: FnOnce(&Context) -> Result<(), cairo::Error>,
{
    let mut surface = ImageSurface::create(Format::ARgb32, width, height).ok()?;
    {
        let context = Context::new(&surface).ok()?;
        context.scale(scale.x, scale.y);
        context.translate(-(origin.0 as f64) / scale.x, -(origin.1 as f64) / scale.y);
        draw_fn(&context).ok()?;
    }
    surface.flush();
    surface.data().ok().map(|data| data.to_vec())
}
