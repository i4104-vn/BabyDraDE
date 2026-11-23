//! Use case: pure window layout policies.
//!
//! Smithay geometry values are used as plain data. All protocol side effects
//! (mapping, configuring, focusing) stay in the compositor adapter.

use smithay::utils::{Logical, Point, Rectangle, Size};

/// Returns the fallback position for a window with no saved geometry.
pub(crate) fn default_position() -> Point<i32, Logical> {
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
    output: Rectangle<i32, Logical>,
    non_exclusive: Rectangle<i32, Logical>,
) -> Rectangle<i32, Logical> {
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
    saved: Option<(Point<i32, Logical>, Size<i32, Logical>)>,
    current_size: Size<i32, Logical>,
) -> (Point<i32, Logical>, Size<i32, Logical>) {
    saved.unwrap_or((default_position(), current_size))
}

#[cfg(test)]
#[path = "../tests/application/windows.rs"]
mod tests;
