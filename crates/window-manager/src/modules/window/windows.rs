// Inlined from window/windows.rs.
//! Window layout use cases.
//!
//! Geometry is pure domain data expressed with [`crate::shared::types::geometry`]
//! value types. All protocol side effects (mapping, configuring, focusing)
//! stay in the compositor adapter.


use crate::shared::types::geometry::{Point, Rectangle, Size};

/// Returns the fallback position for a window with no saved geometry.
pub(crate) fn default_position() -> Point<i32> {
    Point::from((48, 48))
}

/// Computes the target rectangle for a maximized window.
///
/// # Arguments
///
/// * `output` - Output geometry in logical coordinates.
/// * `non_exclusive` - Area left free by layer-shell surfaces.
///
/// # Returns
///
/// The rectangle the window should occupy.
pub(crate) fn maximize_target(
    output: Rectangle<i32>,
    non_exclusive: Rectangle<i32>,
) -> Rectangle<i32> {
    Rectangle::new(output.loc + non_exclusive.loc, non_exclusive.size)
}

/// Computes restore geometry from a saved floating state.
///
/// # Arguments
///
/// * `saved` - Saved location and size, if any.
/// * `current_size` - Current window size used as fallback.
///
/// # Returns
///
/// The location and size to restore.
pub(crate) fn restore_target(
    saved: Option<(Point<i32>, Size<i32>)>,
    current_size: Size<i32>,
) -> (Point<i32>, Size<i32>) {
    saved.unwrap_or((default_position(), current_size))
}

pub(crate) use crate::modules::window::models::windows::WindowAction;

#[cfg(test)]
mod tests {
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

}
