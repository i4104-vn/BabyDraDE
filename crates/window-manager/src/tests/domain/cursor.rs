use super::*;

#[test]
fn edges_map_to_directional_icons() {
    assert_eq!(
        resize_cursor_icon(ResizeEdge::TopLeft),
        CursorIcon::NwResize
    );
    assert_eq!(resize_cursor_icon(ResizeEdge::Right), CursorIcon::EResize);
    assert_eq!(resize_cursor_icon(ResizeEdge::None), CursorIcon::AllResize);
}
