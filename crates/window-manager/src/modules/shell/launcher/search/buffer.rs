//! Off-screen pixel work for the search overlay. Only the panel (plus its
//! shadow) and the full-screen dimmer are allocated — never a second composited
//! surface per result row.

use cairo::{Context, Format, ImageSurface};
use common::constants::launcher::SEARCH_BUFFER_PAD;
use smithay::utils::{Buffer, Rectangle, Scale};

use super::layout::SearchLayout;
use super::SearchRegion;

/// Physical region covering the panel and its soft shadow.
pub(crate) fn panel_region(
    layout: &SearchLayout,
    scale: Scale<f64>,
    screen_w: i32,
    screen_h: i32,
) -> SearchRegion {
    let pad = SEARCH_BUFFER_PAD;
    let x0 = ((layout.x - pad) * scale.x)
        .floor()
        .clamp(0.0, screen_w as f64) as i32;
    let y0 = ((layout.y - pad) * scale.y)
        .floor()
        .clamp(0.0, screen_h as f64) as i32;
    let x1 = ((layout.x + layout.width + pad) * scale.x)
        .ceil()
        .clamp(0.0, screen_w as f64) as i32;
    let y1 = ((layout.y + layout.height + pad) * scale.y)
        .ceil()
        .clamp(0.0, screen_h as f64) as i32;

    (x0, y0, (x1 - x0).max(1), (y1 - y0).max(1))
}

/// Damage rectangle for a freshly written buffer.
pub(crate) fn buffer_damage(width: i32, height: i32) -> Vec<Rectangle<i32, Buffer>> {
    vec![Rectangle::<i32, Buffer>::from_size((width, height).into())]
}

/// Renders the uniform full-screen dimmer. `alpha` is the presence-scaled
/// opacity of the search overlay.
pub(crate) fn render_backdrop_pixels(
    width: i32,
    height: i32,
    scale: Scale<f64>,
    alpha: f64,
) -> Option<Vec<u8>> {
    let mut surface = ImageSurface::create(Format::ARgb32, width, height).ok()?;
    {
        let context = Context::new(&surface).ok()?;
        context.scale(scale.x, scale.y);
        context.set_source_rgba(0.0, 0.0, 0.0, alpha.clamp(0.0, 1.0));
        context.paint().ok()?;
    }
    surface.flush();
    surface.data().ok().map(|data| data.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_region_covers_the_shadow_and_stays_on_screen() {
        let layout = SearchLayout {
            x: 100.0,
            y: 200.0,
            width: 640.0,
            height: 500.0,
            input_x: 0.0,
            input_y: 0.0,
            input_w: 0.0,
            input_h: 0.0,
            rows_x: 0.0,
            rows_y: 0.0,
            rows_w: 0.0,
        };
        let (x, y, w, h) = panel_region(&layout, Scale::from(1.0), 1920, 1080);
        let pad = SEARCH_BUFFER_PAD as i32;
        assert_eq!(x, 100 - pad);
        assert_eq!(y, 200 - pad);
        assert!(w >= 640 && h >= 500);
        assert!(x + w <= 1920 && y + h <= 1080);
    }
}
