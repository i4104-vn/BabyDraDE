use super::*;

fn test_layout(panel: Rect, preview: Rect) -> SwitcherLayout {
    SwitcherLayout {
        panel: ui_kit::components::dock::DockLayout {
            x: panel.x,
            y: panel.y,
            width: panel.width,
            height: panel.height,
            center_y: panel.y + panel.height / 2.0,
        },
        selected: 0,
        preview,
        preview_alpha: 1.0,
    }
}

#[test]
fn overlay_region_covers_capsule_and_preview() {
    let layout = test_layout(
        Rect::new(1848.0, 32.0, 72.0, 1016.0),
        Rect::new(1534.0, 400.0, 300.0, 176.0),
    );

    let region = overlay_region(&layout, Scale::from(1.0), 1920, 1080);
    assert_eq!(region, (1510, 8, 410, 1064));
}

#[test]
fn overlay_region_clamps_to_screen() {
    let layout = test_layout(
        Rect::new(24.0, 32.0, 72.0, 1016.0),
        Rect::new(8.0, 400.0, 300.0, 176.0),
    );
    let region = overlay_region(&layout, Scale::from(1.0), 300, 1000);
    assert_eq!(region.0, 0);
    assert!(region.2 <= 300);
    assert!(region.3 <= 1000);
}

#[test]
fn dock_buffer_maps_logical_coordinates_to_the_region_origin() {
    let pixels = render_pixels(8, 8, Scale::from(1.0), (4, 6), |ctx| {
        ctx.set_source_rgba(1.0, 0.0, 0.0, 1.0);
        ctx.rectangle(5.0, 7.0, 2.0, 2.0);
        ctx.fill()
    })
    .expect("cairo surface");

    // ARGB32 little-endian: byte 3 of each pixel is alpha. Device pixel
    // (0,0) maps to logical (4,6), outside the red rect; (1,1) is inside.
    assert_eq!(pixels[3], 0);
    assert_eq!(pixels[(8 + 1) * 4 + 3], 255);
}
