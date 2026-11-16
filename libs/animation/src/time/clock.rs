use std::time::Instant;

/// Monotonic microsecond tick since `started_at` at current wall-clock time.
#[inline]
pub fn tick_since(started_at: Instant) -> u64 {
    tick_between(started_at, Instant::now())
}

/// Monotonic microsecond tick between `started_at` and `now`.
#[inline]
pub fn tick_between(started_at: Instant, now: Instant) -> u64 {
    now.saturating_duration_since(started_at)
        .as_micros()
        .min(u64::MAX as u128) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_tick_is_monotonic_and_microsecond_grained() {
        let started = Instant::now();
        let first = tick_since(started);
        let second = tick_since(started);

        assert!(second >= first);
    }

    #[test]
    fn tick_between_is_consistent() {
        let started = Instant::now();
        let later = started + std::time::Duration::from_micros(500);
        assert_eq!(tick_between(started, later), 500);
    }
}
