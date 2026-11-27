//! Cairo drawing regression tests for the launcher.

#[allow(unused_imports)]
use cairo::Context;
#[allow(unused_imports)]
use ui_kit::components::dock::{draw_dock, DockLayout, DockSide};
#[allow(unused_imports)]
use ui_kit::drawing::{fit_text, set_bold_font};

#[allow(unused_imports)]
use super::backdrop::render_backdrop_pixels;
#[allow(unused_imports)]
use super::LABEL_FONT_SIZE;

#[test]
fn fit_text_adds_an_ellipsis_when_too_wide() {
    let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 8, 8).expect("surface");
    let context = Context::new(&surface).expect("context");
    set_bold_font(&context, LABEL_FONT_SIZE);

    let fitted = fit_text(&context, "A very long application name", 40.0).expect("text");
    assert!(fitted.ends_with('…'));
    assert!(context.text_extents(&fitted).expect("extents").x_advance() <= 40.0);
}

#[test]
fn draws_shared_dock_surface() {
    let surface =
        cairo::ImageSurface::create(cairo::Format::ARgb32, 200, 600).expect("surface");
    let context = Context::new(&surface).expect("context");
    let panel = DockLayout::new(7, 300.0, 200.0, DockSide::Left);
    assert!(draw_dock(
        &context,
        &panel,
        7,
        3.0,
        3.0,
        Some(3),
        1.0,
        0.0,
        DockSide::Left,
        |_, _, _, _, _, _| Ok(()),
    )
    .is_ok());
}

#[test]
fn renders_backdrop_gradient() {
    let scale = smithay::utils::Scale::from((1.0, 1.0));
    let pixels = render_backdrop_pixels(400, 800, scale, 400.0, 1.0);
    assert!(pixels.is_some());
    assert_eq!(pixels.unwrap().len(), 400 * 800 * 4);
}
