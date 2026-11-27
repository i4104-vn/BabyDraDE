//! Pluggable accelerometer drivers and the factory that selects them.
//!
//! A driver only knows how to produce a gravity sample. The factory owns an
//! ordered list of candidates, supports explicit selection from configuration,
//! and the manager replaces a driver that stops producing readings. Adding a
//! driver for a new machine means registering one probe function; no consumer
//! code changes.

use std::time::Duration;

use super::Gravity;

mod factory;
mod manager;
mod selection;

#[cfg(test)]
mod tests;

pub use factory::DriverFactory;
pub use manager::DriverManager;
pub use selection::DriverSelection;

/// Sampling interval used when a driver does not request its own.
pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(250);

/// A source of gravity samples.
///
/// Drivers are polled on a dedicated worker thread, so they must be movable
/// across threads.
pub trait RotationDriver: Send {
    /// Identifier used in logs and configuration (`iio`, `sensor-proxy`, …).
    fn name(&self) -> &str;

    /// Reads the current acceleration; `None` means a transient failure.
    fn read(&mut self) -> Option<Gravity>;

    /// Sampling interval preferred by this driver.
    fn poll_interval(&self) -> Duration {
        DEFAULT_POLL_INTERVAL
    }
}
