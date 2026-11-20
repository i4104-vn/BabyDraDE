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
/// GTK/GDK frame-clock animation helpers shared by shell features.
pub mod gtk;
pub mod spring;
pub mod time;
pub mod tween;

// Re-exports for root access and full backward compatibility
pub use curves::Easing;
pub use gtk::{
    current_anim_gen, island_animate_size, island_animate_width, island_zoom_in, island_zoom_out,
    next_anim_gen, slide_in, slide_in_cancelable, slide_out, slide_out_cb, slide_out_cb_cancelable,
    SlideDirection,
};
pub use spring::{Spring, SpringConfig};
pub use time::{tick_between, tick_since, DEFAULT_DURATION, FRAME_INTERVAL, TARGET_FPS};
pub use tween::Tween;
