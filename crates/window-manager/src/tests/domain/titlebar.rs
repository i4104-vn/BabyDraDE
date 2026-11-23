use super::*;

#[test]
fn layouts_share_button_positions() {
    let layout = TitlebarLayout::new(800.0, false);
    let close = layout.button(Action::Close);
    let max = layout.button(Action::Maximize);
    let min = layout.button(Action::Minimize);
    assert_eq!(close.x - max.x, 24.0);
    assert_eq!(max.x - min.x, 24.0);
    assert_eq!(close.y, max.y);
    assert_eq!(max.y, min.y);
}

#[test]
fn hit_test_matches_layout_buttons() {
    let frame = FrameInput {
        location: Point::from((100, 200)),
        width: 800.0,
        maximized: false,
        server_side_decoration: true,
        fullscreen: false,
    };
    let close = TitlebarLayout::new(frame.width, false).button(Action::Close);
    let point = Point::from((frame.location.x as f64 + close.x, 180.0));
    assert_eq!(hit_test(frame, point), Some(Action::Close));
}

#[test]
fn client_decorations_and_fullscreen_have_no_frame() {
    let mut frame = FrameInput {
        location: Point::from((0, 0)),
        width: 800.0,
        maximized: false,
        server_side_decoration: false,
        fullscreen: false,
    };
    assert_eq!(hit_test(frame, Point::from((750.0, 10.0))), None);
    frame.server_side_decoration = true;
    frame.fullscreen = true;
    assert_eq!(hit_test(frame, Point::from((750.0, 10.0))), None);
}
