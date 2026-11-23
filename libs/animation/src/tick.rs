//! Cache-key helpers for animating surfaces.
//!
//! Renderers invalidate off-screen buffers by comparing a tick derived from
//! the animation clock. A tween that is at rest has no meaningful tick, so it
//! reports [`SETTLED_TICK`]; only tweens that are still moving contribute.

use crate::tween::Tween;

/// Sentinel meaning "nothing is animating" — safe to use inside a cache key.
pub const SETTLED_TICK: u64 = u64::MAX;

impl Tween {
    /// Tick to use in a cache key: the current frame tick while animating,
    /// [`SETTLED_TICK`] once the tween rests.
    #[inline]
    pub fn cache_tick(&self) -> u64 {
        if self.is_animating() {
            self.frame_tick()
        } else {
            SETTLED_TICK
        }
    }
}

/// Aggregates the ticks of several tweens into one cache-key component.
///
/// Returns the most recent tick among the tweens that are still animating, or
/// [`SETTLED_TICK`] when every tween is at rest. Settled tweens are filtered out
/// first, otherwise their sentinel would mask a live animation.
///
/// ```
/// use animation::{animation_tick, SETTLED_TICK, Tween};
///
/// assert_eq!(animation_tick([&Tween::settled(0.0)]), SETTLED_TICK);
/// ```
#[inline]
pub fn animation_tick<'a>(tweens: impl IntoIterator<Item = &'a Tween>) -> u64 {
    tweens
        .into_iter()
        .filter(|tween| tween.is_animating())
        .map(|tween| tween.frame_tick())
        .max()
        .unwrap_or(SETTLED_TICK)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Easing;
    use std::time::Duration;

    #[test]
    fn a_single_settled_tween_reports_the_sentinel() {
        let tween = Tween::settled(1.0);
        assert_eq!(tween.cache_tick(), SETTLED_TICK);
        assert_eq!(animation_tick([&tween]), SETTLED_TICK);
    }

    #[test]
    fn animating_tweens_override_the_sentinel() {
        let moving = Tween::new(0.0, 1.0, Duration::from_secs(10), Easing::Linear);
        let resting = Tween::settled(1.0);
        let tick = animation_tick([&resting, &moving]);
        // Both are live microsecond ticks; they need not be byte-identical
        // because `frame_tick` reads the clock on every call.
        assert!(tick < SETTLED_TICK);
        assert!(moving.cache_tick() < SETTLED_TICK);
    }
}
