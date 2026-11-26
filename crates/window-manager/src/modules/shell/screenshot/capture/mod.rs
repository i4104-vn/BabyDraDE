//! Direct compositor screenshot capture — no external capture utilities.

use crate::modules::shell::screenshot::models::CaptureState;

/// Shared flag set by the keymap thread and consumed by the render loops.
static CAPTURE_REQUESTED: CaptureState = CaptureState::new();

/// Converts smithay's `ReadPixels` output into cairo's `ARgb32` buffer.
///
/// Smithay's GLES projection includes a `flip180` factor, so rows arrive
/// top-down already; do **not** reverse them or the saved PNG ends up mirrored
/// (and rotated incorrectly on transformed outputs). Only RGBA -> BGRA needs
/// swapping on little-endian hosts.
fn rgba_to_argb32(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut bgra = vec![0u8; rgba.len()];
    let row = width * 4;
    for y in 0..height {
        let src = &rgba[y * row..(y + 1) * row];
        let dst = &mut bgra[y * row..(y + 1) * row];
        for x in 0..width {
            let (s, d) = (&src[x * 4..x * 4 + 4], &mut dst[x * 4..x * 4 + 4]);
            d[0] = s[2];
            d[1] = s[1];
            d[2] = s[0];
            d[3] = s[3];
        }
    }
    bgra
}

/// Requests a screenshot. Safe to call from any thread.
pub(crate) fn trigger() {
    CAPTURE_REQUESTED.request();
}

/// Consumes a pending screenshot request. Called once per rendered frame.
pub(crate) fn take_requested() -> bool {
    CAPTURE_REQUESTED.take()
}

pub(crate) use render::capture_screen;

mod render;

mod transform;

mod tests {
    #[allow(unused_imports)]
    use smithay::utils::{Size, Transform};

    #[allow(unused_imports)]
    use super::transform::transform_matrix;
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use smithay::utils::{Physical, Point};

    #[test]
    fn rgba_conversion_keeps_rows_top_down() {
        // Row 0 of ReadPixels is the top row of the image. A vertical flip here
        // used to mirror the saved screenshot (and break rotated outputs).
        let width = 2;
        let height = 2;
        let rgba = vec![
            255, 0, 0, 255, 0, 255, 0, 255, // top row: red, green
            0, 0, 255, 255, 255, 255, 255, 255, // bottom row: blue, white
        ];
        let bgra = rgba_to_argb32(&rgba, width, height);

        // Top row stays first, channels are swapped to BGRA.
        assert_eq!(&bgra[0..4], &[0, 0, 255, 255]);
        assert_eq!(&bgra[4..8], &[0, 255, 0, 255]);
        assert_eq!(&bgra[8..12], &[255, 0, 0, 255]);
        assert_eq!(&bgra[12..16], &[255, 255, 255, 255]);
    }

    #[test]
    fn transform_matrix_matches_smithay_point_mapping() {
        let (w, h) = (13, 7);
        let area = Size::<i32, Physical>::from((w, h));
        let transforms = [
            Transform::Normal,
            Transform::_90,
            Transform::_180,
            Transform::_270,
            Transform::Flipped,
            Transform::Flipped90,
            Transform::Flipped180,
            Transform::Flipped270,
        ];

        for transform in transforms {
            let (a, b, c, d, tx, ty) = transform_matrix(transform, w, h);
            for x in 0..=w {
                for y in 0..=h {
                    let expected =
                        transform.transform_point_in(Point::<i32, Physical>::from((x, y)), &area);
                    let actual = (
                        (a * x as f64 + c * y as f64 + tx) as i32,
                        (b * x as f64 + d * y as f64 + ty) as i32,
                    );
                    assert_eq!(
                        actual,
                        (expected.x, expected.y),
                        "transform {:?}",
                        transform
                    );
                }
            }
        }
    }
}
