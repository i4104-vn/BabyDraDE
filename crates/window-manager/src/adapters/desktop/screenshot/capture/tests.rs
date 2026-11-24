use smithay::utils::{Size, Transform};

use super::transform::transform_matrix;
use super::*;
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
