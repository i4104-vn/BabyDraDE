//! Pure margin math and generation tokens for frame-synced slide transitions.

use std::cell::Cell;
use std::rc::Rc;

/// Whether a widget stays visible after a slide-out finishes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VisibilityAfter {
    /// Leave the widget visible (the caller decides what happens next).
    #[default]
    Keep,
    /// Hide the widget when the animation completes.
    Hide,
}

impl VisibilityAfter {
    pub(crate) fn hides(self) -> bool {
        matches!(self, Self::Hide)
    }
}

/// A token identifying one generation of an animation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GenerationToken(u64);

impl GenerationToken {
    /// Wraps a caller-owned generation number.
    pub fn new(value: u64) -> Self {
        Self(value)
    }
}

/// Shared generation counter used to cancel superseded slide animations.
///
/// Wrapping the counter keeps `Rc<Cell<_>>` out of the public API while still
/// letting callers observe the generation in their completion callbacks.
#[derive(Clone)]
pub struct AnimationGeneration(Rc<Cell<u64>>);

impl Default for AnimationGeneration {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationGeneration {
    /// Creates a fresh generation counter.
    pub fn new() -> Self {
        Self(Rc::new(Cell::new(0)))
    }

    /// Publishes the next generation and returns its token.
    pub fn advance(&self) -> GenerationToken {
        let next = self.0.get().wrapping_add(1);
        self.0.set(next);
        GenerationToken(next)
    }

    /// Whether `token` still identifies the newest animation.
    pub fn is_current(&self, token: GenerationToken) -> bool {
        self.0.get() == token.0
    }

    /// Adapts an externally owned counter used by the legacy entry points.
    pub(crate) fn from_cell(cell: Rc<Cell<u64>>) -> Self {
        Self(cell)
    }
}

/// Direction axis for a slide transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlideDirection {
    Down,
    Up,
    Left,
    Right,
}

/// Initial margin of a widget entering from a given direction.
///
/// # Arguments
/// * `original` - Margin the widget rests at.
/// * `distance` - Travel distance in pixels.
///
/// # Returns
/// The clamped margin at the start of the transition.
pub(crate) fn start_margin(original: i32, distance: i32) -> i32 {
    (original - distance).max(0)
}

/// Margin of an entering widget at a given eased progress.
///
/// # Arguments
/// * `original` - Margin the widget rests at.
/// * `distance` - Travel distance in pixels.
/// * `eased` - Eased progress in `0.0..=1.0`.
///
/// # Returns
/// The interpolated, non-negative margin.
pub(crate) fn entering_margin(original: i32, distance: i32, eased: f64) -> i32 {
    (original - distance + (distance as f64 * eased) as i32).max(0)
}

/// Margin of an exiting widget at a given eased progress.
///
/// # Arguments
/// * `original` - Margin the widget rests at.
/// * `distance` - Travel distance in pixels.
/// * `eased` - Eased progress in `0.0..=1.0`.
///
/// # Returns
/// The interpolated, non-negative margin.
pub(crate) fn exiting_margin(original: i32, distance: i32, eased: f64) -> i32 {
    (original - (distance as f64 * eased) as i32).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entering_margin_reaches_rest_at_full_progress() {
        assert_eq!(entering_margin(40, 30, 1.0), 40);
        assert_eq!(entering_margin(40, 30, 0.0), start_margin(40, 30));
    }

    #[test]
    fn margins_never_become_negative_on_large_distances() {
        assert_eq!(start_margin(10, 50), 0);
        assert_eq!(entering_margin(10, 50, 0.5), 0);
        assert_eq!(entering_margin(10, 50, 1.0), 10);
        assert_eq!(exiting_margin(10, 50, 0.5), 0);
    }

    #[test]
    fn exiting_margin_decreases_monotonically() {
        let mut previous = i32::MAX;
        for step in 0..=10 {
            let margin = exiting_margin(100, 40, step as f64 / 10.0);
            assert!(margin <= previous);
            previous = margin;
        }
        assert_eq!(previous, 60);
    }

    #[test]
    fn superseded_generations_are_detected() {
        let generation = AnimationGeneration::new();
        let first = generation.advance();
        assert!(generation.is_current(first));
        let second = generation.advance();
        assert!(!generation.is_current(first));
        assert!(generation.is_current(second));
    }
}
