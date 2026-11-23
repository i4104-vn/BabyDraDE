//! Maps Wayland resize edges to Smithay cursor icons.
//!
//! This is protocol/toolkit knowledge and therefore belongs in an adapter,
//! not in `domain`.

use smithay::{
    input::pointer::CursorIcon,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
};

/// Returns the cursor shape that matches the resize edge being dragged.
pub(crate) fn resize_cursor_icon(edges: ResizeEdge) -> CursorIcon {
    use ResizeEdge::*;
    match edges {
        TopLeft => CursorIcon::NwResize,
        TopRight => CursorIcon::NeResize,
        BottomLeft => CursorIcon::SwResize,
        BottomRight => CursorIcon::SeResize,
        Top => CursorIcon::NResize,
        Bottom => CursorIcon::SResize,
        Left => CursorIcon::WResize,
        Right => CursorIcon::EResize,
        _ => CursorIcon::AllResize,
    }
}

#[cfg(test)]
mod tests {
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
}
