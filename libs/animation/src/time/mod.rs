pub mod clock;
pub mod constants;

pub use clock::{tick_between, tick_since};
pub use constants::{DEFAULT_DURATION, FRAME_INTERVAL, TARGET_FPS};
