use std::time::Duration;

/// Frame rate the compositor's animation loops aim for.
pub const TARGET_FPS: u32 = 120;

/// Duration of one frame at [`TARGET_FPS`] (about 8.33 ms).
pub const FRAME_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS as u64);

/// Default duration for UI transitions.
pub const DEFAULT_DURATION: Duration = Duration::from_millis(420);

/// Smooth highlight/selection step, e.g. keyboard navigation in a dock.
pub const NAVIGATION_DURATION: Duration = Duration::from_millis(200);

/// Short glide when the highlight moves between icons (pointer hover).
pub const HOVER_DURATION: Duration = Duration::from_millis(140);

/// Duration of a wheel/touchpad scroll step.
pub const SCROLL_DURATION: Duration = Duration::from_millis(260);

/// Overlay fade-in duration.
pub const FADE_IN_DURATION: Duration = Duration::from_millis(320);

/// Overlay fade-out duration.
pub const FADE_OUT_DURATION: Duration = Duration::from_millis(260);

/// Wallpaper circular-reveal duration.
pub const REVEAL_DURATION: Duration = Duration::from_millis(900);

/// Window open / close / minimize / restore / geometry transitions.
pub const WINDOW_OPEN_DURATION: Duration = Duration::from_millis(280);
pub const WINDOW_CLOSE_DURATION: Duration = Duration::from_millis(260);
pub const WINDOW_MINIMIZE_DURATION: Duration = Duration::from_millis(280);
pub const WINDOW_RESTORE_DURATION: Duration = Duration::from_millis(300);
pub const WINDOW_GEOMETRY_DURATION: Duration = Duration::from_millis(340);

/// Alt+Tab switcher stage pulse, dock highlight and preview/scroll steps.
pub const STAGE_ANIM_DURATION: Duration = Duration::from_millis(500);
pub const DOCK_TARGET_ANIM_DURATION: Duration = Duration::from_millis(420);
pub const SWITCHER_PREVIEW_DURATION: Duration = Duration::from_millis(180);
pub const SWITCHER_SCROLL_DURATION: Duration = Duration::from_millis(160);

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
