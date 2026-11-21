pub mod clock;
pub mod constants;
pub mod frame;

pub use clock::{tick_between, tick_since};
pub use constants::{
    DEFAULT_DURATION, FADE_IN_DURATION, FADE_OUT_DURATION, FRAME_INTERVAL, NAVIGATION_DURATION,
    REVEAL_DURATION, SCROLL_DURATION, TARGET_FPS,
};
pub use frame::{bounded_delta, frame_index_since};
