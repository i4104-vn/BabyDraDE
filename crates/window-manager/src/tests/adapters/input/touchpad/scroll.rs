use super::*;
use smithay::backend::input::AxisRelativeDirection;

#[test]
fn test_apply_natural_scroll_passthrough_when_already_inverted() {
    let (amt, v120, dir) = apply_natural_scroll(
        true,
        AxisRelativeDirection::Inverted,
        Some(-15.0),
        Some(-120.0),
    );
    assert_eq!(amt, Some(-15.0));
    assert_eq!(v120, Some(-120.0));
    assert_eq!(dir, AxisRelativeDirection::Inverted);
}

#[test]
fn test_apply_natural_scroll_software_inverts_when_enabled() {
    let (amt, v120, dir) = apply_natural_scroll(
        true,
        AxisRelativeDirection::Identical,
        Some(15.0),
        Some(120.0),
    );
    assert_eq!(amt, Some(-15.0));
    assert_eq!(v120, Some(-120.0));
    assert_eq!(dir, AxisRelativeDirection::Inverted);
}

#[test]
fn test_apply_natural_scroll_identical_when_disabled() {
    let (amt, v120, dir) = apply_natural_scroll(
        false,
        AxisRelativeDirection::Identical,
        Some(15.0),
        Some(120.0),
    );
    assert_eq!(amt, Some(15.0));
    assert_eq!(v120, Some(120.0));
    assert_eq!(dir, AxisRelativeDirection::Identical);
}

#[test]
fn test_apply_natural_scroll_un_inverts_when_disabled_but_hw_inverted() {
    let (amt, v120, dir) = apply_natural_scroll(
        false,
        AxisRelativeDirection::Inverted,
        Some(-15.0),
        Some(-120.0),
    );
    assert_eq!(amt, Some(15.0));
    assert_eq!(v120, Some(120.0));
    assert_eq!(dir, AxisRelativeDirection::Identical);
}
