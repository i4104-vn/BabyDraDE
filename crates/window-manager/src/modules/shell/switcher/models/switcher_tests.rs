//! Tests for switcher navigation wrapping.

#[allow(unused_imports)]
use super::step_target;

#[test]
fn dock_wraps_directly_to_the_first_or_last_icon() {
    assert_eq!(step_target(0.0, -1.0, 5), (4.0, true));
    assert_eq!(step_target(0.0, 1.0, 5), (1.0, false));
    assert_eq!(step_target(4.0, 1.0, 5), (0.0, true));
    assert_eq!(step_target(4.0, -1.0, 5), (3.0, false));
}
