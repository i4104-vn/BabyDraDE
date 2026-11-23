use super::*;

#[test]
fn maximize_uses_the_non_exclusive_area_inside_the_output() {
    let output = Rectangle::new(Point::from((100, 50)), Size::from((1920, 1080)));
    let usable = Rectangle::new(Point::from((0, 40)), Size::from((1920, 1000)));
    assert_eq!(
        maximize_target(output, usable),
        Rectangle::new(Point::from((100, 90)), Size::from((1920, 1000)))
    );
}

#[test]
fn restore_prefers_saved_geometry_and_falls_back_to_current_size() {
    let current = Size::from((640, 480));
    let saved_loc = Point::from((10, 20));
    let saved_size = Size::from((800, 600));
    assert_eq!(
        restore_target(Some((saved_loc, saved_size)), current),
        (saved_loc, saved_size)
    );
    assert_eq!(
        restore_target(None, current),
        (Point::from((48, 48)), current)
    );
}
