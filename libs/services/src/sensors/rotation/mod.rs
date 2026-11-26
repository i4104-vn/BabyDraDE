//! Device orientation sensing: pluggable accelerometer drivers and the
//! debounce tracker that turns raw samples into quarter-turn steps.
//!
//! Kept in `services` so the compositor and the pre-login greeter share one
//! implementation instead of duplicating sensor and D-Bus logic.

mod driver;
mod drivers;
mod tracker;

pub use driver::{
    DriverFactory, DriverManager, DriverSelection, RotationDriver, DEFAULT_POLL_INTERVAL,
};
pub use tracker::{Gravity, Orientation, RotationTracker, REQUIRED_STABLE_READINGS};
