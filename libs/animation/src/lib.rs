//! Frame-rate independent animation primitives shared by BabyDraDE UI and compositor.
//!
//! Every animation is driven by wall-clock time, so its result does not depend
//! on how often a renderer runs.
//!
//! # Primitives
//! - [`Tween`]: Time-budgeted interpolation using rich [`Easing`] curves.
//! - [`Spring`]: Physics-driven damped harmonic oscillator with momentum preservation.
//!
//! ```
//! use animation::{Easing, Tween, DEFAULT_DURATION};
//!
//! let mut fade = Tween::settled(0.0);
//! fade.retarget(1.0, DEFAULT_DURATION, Easing::EaseOutBack);
//! assert!((0.0..=1.5).contains(&fade.value()));
//! ```
//!
//! ```
//! use animation::{Spring, SpringConfig};
//!
//! let mut bouncy = Spring::new(0.0, 100.0, SpringConfig::BOUNCY);
//! assert!(bouncy.is_animating());
//! ```

pub mod curves;
pub mod spring;
pub mod time;
pub mod tween;

// Re-exports for root access and full backward compatibility.
// GTK frame-clock helpers live in the separate `animation-gtk` crate so this
// crate stays usable without a UI toolkit.
pub use curves::Easing;
pub use spring::{Spring, SpringConfig};
pub use time::{
    bounded_delta, frame_index_since, tick_between, tick_since, DEFAULT_DURATION,
    DOCK_TARGET_ANIM_DURATION, FADE_IN_DURATION, FADE_OUT_DURATION, FRAME_INTERVAL, HOVER_DURATION,
    NAVIGATION_DURATION, REVEAL_DURATION, SCROLL_DURATION, SEARCH_FLY_DURATION,
    SEARCH_OPEN_DURATION, STAGE_ANIM_DURATION, SWITCHER_PREVIEW_DURATION, SWITCHER_SCROLL_DURATION,
    TARGET_FPS, WINDOW_CLOSE_DURATION, WINDOW_GEOMETRY_DURATION, WINDOW_MINIMIZE_DURATION,
    WINDOW_OPEN_DURATION, WINDOW_RESTORE_DURATION,
};
pub use tween::Tween;
