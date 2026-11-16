//! Frame-rate independent animation primitives shared by BabyDraDE UI and compositor.
//!
//! Every animation is driven by wall-clock time, so its result does not depend
//! on how often a renderer runs. The frame-rate constants describe the frame
//! budget that animation loops should use; they never cap the animation itself.
//!
//! ```
//! use animation::{Easing, Tween, DEFAULT_DURATION};
//!
//! let mut fade = Tween::settled(0.0);
//! fade.retarget(1.0, DEFAULT_DURATION, Easing::EaseOutCubic);
//!
//! // The value is derived from the clock, so a 60 Hz renderer, a 120 Hz
//! // renderer and a stuttering renderer all reach the same end state.
//! assert!((0.0..=1.0).contains(&fade.value()));
//! ```

pub mod clock;
pub mod constants;
pub mod easing;
pub mod tween;

pub use clock::{tick_between, tick_since};
pub use constants::{DEFAULT_DURATION, FRAME_INTERVAL, TARGET_FPS};
pub use easing::Easing;
pub use tween::Tween;
