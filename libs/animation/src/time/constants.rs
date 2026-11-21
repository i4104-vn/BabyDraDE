use std::time::Duration;

/// Frame rate the compositor's animation loops aim for.
pub const TARGET_FPS: u32 = 120;

/// Duration of one frame at [`TARGET_FPS`] (about 8.33 ms).
pub const FRAME_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS as u64);

/// Default duration for UI transitions.
pub const DEFAULT_DURATION: Duration = Duration::from_millis(420);

/// Quick highlight/selection step, e.g. keyboard navigation in a dock.
pub const NAVIGATION_DURATION: Duration = Duration::from_millis(220);

/// Duration of a wheel/touchpad scroll step.
pub const SCROLL_DURATION: Duration = Duration::from_millis(180);

/// Overlay fade-in duration.
pub const FADE_IN_DURATION: Duration = Duration::from_millis(240);

/// Overlay fade-out duration.
pub const FADE_OUT_DURATION: Duration = Duration::from_millis(180);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_frame_rate_is_120_fps() {
        assert_eq!(TARGET_FPS, 120);
        assert!(FRAME_INTERVAL >= Duration::from_micros(8_333));
        assert!(FRAME_INTERVAL <= Duration::from_micros(8_334));
        assert!(DEFAULT_DURATION >= Duration::from_millis(100));
    }
}
