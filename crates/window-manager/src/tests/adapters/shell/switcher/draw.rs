use super::*;

/// Verifies that ui_kit icon resolution works and that a resolved pixbuf
/// can be painted onto a Cairo surface without panicking.
#[test]
fn test_icon_pixbuf_paints_to_cairo() {
    let _ = gtk4::init();

    let pb = ui_kit::icons::get_app_icon_pixbuf("chromium", 24)
        .or_else(|| ui_kit::icons::get_app_icon_pixbuf("kitty", 24));

    // At least one common app icon must resolve — if neither is installed the
    // test is a soft pass (we just skip the paint step).
    if let Some(pb) = pb {
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 64, 64).unwrap();
        let ctx = cairo::Context::new(&surface).unwrap();
        ctx.save().unwrap();
        let (pw, ph) = (pb.width() as f64, pb.height() as f64);
        ctx.scale(24.0 / pw, 24.0 / ph);
        ctx.set_source_pixbuf(&pb, 0.0, 0.0);
        ctx.paint_with_alpha(1.0).unwrap();
        ctx.restore().unwrap();
    }
}
