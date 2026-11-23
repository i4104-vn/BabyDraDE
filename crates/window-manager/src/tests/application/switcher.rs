use super::*;

#[test]
fn scrubs_and_wraps_around_the_list() {
    assert_eq!(scrub_index(0.0, 60.0, 0, 3), 1);
    assert_eq!(scrub_index(0.0, -60.0, 0, 3), 2);
    assert_eq!(scrub_index(0.0, 180.0, 0, 3), 0);
    assert_eq!(scrub_index(0.0, 60.0, 0, 0), 0);
}
