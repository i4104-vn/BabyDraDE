//! Device orientation sensing: pluggable accelerometer drivers and the
//! debounce tracker that turns raw samples into quarter-turn steps.
//!
//! Kept in `services` so the compositor and the pre-login greeter share one
//! implementation instead of duplicating sensor and D-Bus logic.

mod driver;
mod drivers;
mod tracker;
mod worker;

pub use driver::{DriverFactory, DriverManager, DriverSelection, RotationDriver};
pub use tracker::{Gravity, RotationTracker};
pub use worker::RotationWorker;
