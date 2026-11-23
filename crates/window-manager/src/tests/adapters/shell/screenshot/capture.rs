use super::*;
use smithay::utils::{Physical, Point};

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
