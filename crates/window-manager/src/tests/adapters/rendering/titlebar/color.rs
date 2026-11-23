use super::*;

#[test]
fn preserves_a_light_dominant_color() {
    let samples = [
        [0.95, 0.95, 0.95],
        [0.96, 0.96, 0.96],
        [0.94, 0.94, 0.94],
        [0.1, 0.1, 0.1],
    ];
    assert!(dominant_color(&samples).unwrap()[0] > 0.9);
}
