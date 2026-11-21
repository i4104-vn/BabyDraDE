//! Frame-pacing helpers for 120 fps render loops.

use std::time::{Duration, Instant};

use super::constants::FRAME_INTERVAL;

/// Monotonic frame index since `started_at`, quantized to the shared 120 fps
/// budget. Two calls inside the same frame return the same index, which makes
/// it a stable cache key for animation frames.
pub fn frame_index_since(started_at: Instant) -> u64 {
    let elapsed = started_at.elapsed();
    (elapsed.as_nanos() / FRAME_INTERVAL.as_nanos()) as u64
}

/// Advances `previous` to now and returns the delta, capped at four frame
/// budgets so a long stall cannot produce an oversized animation step.
pub fn bounded_delta(previous: &mut Instant) -> Duration {
    let now = Instant::now();
    let delta = now.saturating_duration_since(*previous);
    *previous = now;
    delta.min(FRAME_INTERVAL * 4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_index_is_quantized_to_the_target_frame_rate() {
        // Subtract a hair less than three budgets so timing jitter cannot tip
        // the quantization over the boundary.
        let started = Instant::now() - FRAME_INTERVAL * 3 + Duration::from_micros(1);
        assert_eq!(frame_index_since(started), 2);
    }

    #[test]
    fn bounded_delta_caps_long_stalls() {
        let mut previous = Instant::now() - FRAME_INTERVAL * 20;
        assert_eq!(bounded_delta(&mut previous), FRAME_INTERVAL * 4);
    }
}
