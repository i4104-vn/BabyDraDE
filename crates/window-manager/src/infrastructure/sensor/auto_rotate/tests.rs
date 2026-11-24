use super::models::Orientation;

use super::*;

#[test]
fn gravity_maps_to_quarters() {
    assert_eq!(Orientation::from_gravity(0.0, 1.0), Some(Orientation(0)));
    assert_eq!(Orientation::from_gravity(1.0, 0.0), Some(Orientation(3)));
    assert_eq!(Orientation::from_gravity(0.0, -1.0), Some(Orientation(2)));
    assert_eq!(Orientation::from_gravity(-1.0, 0.0), Some(Orientation(1)));
    assert_eq!(Orientation::from_gravity(0.0, 0.0), None);
}

#[test]
fn rotation_cycles_through_all_quarters() {
    assert_eq!(rotate(Transform::Normal, 1), Transform::_90);
    assert_eq!(rotate(Transform::Normal, 4), Transform::Normal);
    assert_eq!(rotate(Transform::Flipped90, 1), Transform::Flipped180);
}

#[test]
fn orientation_steps_wrap_around() {
    assert_eq!(Orientation(3).steps_from(Orientation(0)), 3);
    assert_eq!(Orientation(0).steps_from(Orientation(3)), 1);
}
