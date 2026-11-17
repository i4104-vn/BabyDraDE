//! Shared enumerations for actions, gestures, and window states.

/// Actions triggered by titlebar buttons or gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Action {
    Close,
    Minimize,
    Maximize,
    Move,
}

/// Gesture swipe directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}

impl SwipeDirection {
    /// Resolves the dominant direction from a 2-D accumulated delta.
    ///
    /// Returns `None` when neither axis exceeds `threshold`.
    pub fn from_delta(dx: f64, dy: f64, threshold: f64) -> Option<Self> {
        let abs_x = dx.abs();
        let abs_y = dy.abs();
        if abs_x < threshold && abs_y < threshold {
            return None;
        }
        if abs_x >= abs_y {
            if dx > 0.0 {
                Some(Self::Right)
            } else {
                Some(Self::Left)
            }
        } else if dy > 0.0 {
            Some(Self::Down)
        } else {
            Some(Self::Up)
        }
    }
}
