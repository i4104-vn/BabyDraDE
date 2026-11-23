use super::*;

#[test]
fn right_backdrop_is_darkest_at_the_right_edge() {
    let pixels = render_right_backdrop_pixels(400, 100, Scale::from(1.0), 400.0, 1.0)
        .expect("backdrop pixels");
    let alpha_at = |x: usize| pixels[(50 * 400 + x) * 4 + 3];

    assert!(alpha_at(390) > 100);
    assert!(alpha_at(10) < 20);
}
