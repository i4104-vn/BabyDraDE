use super::*;

#[test]
fn screenshot_path_creates_directory_and_uses_stamp() {
    let root = tempfile::tempdir().expect("temp root");
    let path = screenshot_path_in(root.path(), "2026-11-24_10-00-00");

    assert_eq!(
        path,
        root.path()
            .join("Screenshots")
            .join("Screenshot_2026-11-24_10-00-00.png")
    );
    assert!(
        path.parent().expect("parent").is_dir(),
        "Screenshots directory should be created"
    );
}

#[test]
fn timestamp_keeps_screenshot_filename_layout() {
    let stamp = timestamp();
    let bytes = stamp.as_bytes();

    assert_eq!(stamp.len(), 19, "expected YYYY-MM-DD_HH-MM-SS, got {stamp}");
    assert_eq!(&bytes[4..5], b"-");
    assert_eq!(&bytes[7..8], b"-");
    assert_eq!(&bytes[10..11], b"_");
    assert_eq!(&bytes[13..14], b"-");
    assert_eq!(&bytes[16..17], b"-");
}

#[test]
fn save_png_roundtrip_preserves_surface() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("shot.png");
    let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 3, 2).expect("surface");
    {
        let cr = cairo::Context::new(&surface).expect("context");
        cr.set_source_rgb(1.0, 0.0, 0.0);
        cr.paint().expect("paint");
    }

    assert!(save_png(&surface, &path));

    let mut file = File::open(&path).expect("open png");
    let decoded = cairo::ImageSurface::create_from_png(&mut file).expect("decode png");
    assert_eq!((decoded.width(), decoded.height()), (3, 2));
}

#[test]
fn save_png_returns_false_when_parent_is_missing() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("missing").join("shot.png");
    let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 1, 1).expect("surface");

    assert!(!save_png(&surface, &path));
    assert!(!path.exists());
}
