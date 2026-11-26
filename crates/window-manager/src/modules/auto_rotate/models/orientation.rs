//! Smithay-specific transform rotation; sensor maths lives in `services`.

use smithay::utils::Transform;

/// Rotates a transform clockwise by `steps` quarter turns.
///
/// Both the plain and flipped families are handled, so a panel mounted
/// upside down keeps its mirror while rotating.
pub(crate) fn rotate(transform: Transform, steps: u8) -> Transform {
    let mut result = transform;
    for _ in 0..steps % 4 {
        result = match result {
            Transform::Normal => Transform::_90,
            Transform::_90 => Transform::_180,
            Transform::_180 => Transform::_270,
            Transform::_270 => Transform::Normal,
            Transform::Flipped => Transform::Flipped90,
            Transform::Flipped90 => Transform::Flipped180,
            Transform::Flipped180 => Transform::Flipped270,
            Transform::Flipped270 => Transform::Flipped,
        };
    }
    result
}

#[cfg(test)]
mod tests {
    use super::rotate;
    use smithay::utils::Transform;

    #[test]
    fn rotate_cycles_through_all_eight_transforms() {
        assert_eq!(rotate(Transform::Normal, 1), Transform::_90);
        assert_eq!(rotate(Transform::Normal, 4), Transform::Normal);
        assert_eq!(rotate(Transform::Flipped90, 1), Transform::Flipped180);
        assert_eq!(rotate(Transform::Flipped270, 1), Transform::Flipped);
    }
}
