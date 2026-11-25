//! Use case: switcher selection math for live finger scrubbing.

/// Logical pixels per switcher slot while dragging.
const SLOT_WIDTH: f64 = 60.0;

/// Maps a horizontal drag distance to a wrapped switcher index.
///
/// # Arguments
///
/// * `start_x` - Pointer X coordinate when the drag began.
/// * `current_x` - Current pointer X coordinate.
/// * `base_index` - Selected entry when the drag began.
/// * `count` - Number of entries in the switcher.
///
/// # Returns
///
/// The wrapped index to select, or `0` for an empty switcher.
pub(crate) fn scrub_index(start_x: f64, current_x: f64, base_index: usize, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    let steps = ((current_x - start_x) / SLOT_WIDTH).round() as i64;
    (base_index as i64 + steps).rem_euclid(count as i64) as usize
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn scrubs_and_wraps_around_the_list() {
        assert_eq!(scrub_index(0.0, 60.0, 0, 3), 1);
        assert_eq!(scrub_index(0.0, -60.0, 0, 3), 2);
        assert_eq!(scrub_index(0.0, 180.0, 0, 3), 0);
        assert_eq!(scrub_index(0.0, 60.0, 0, 0), 0);
    }
}
