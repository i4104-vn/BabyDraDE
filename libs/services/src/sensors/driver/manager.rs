//! Active driver ownership and failure-driven failover.

use std::time::Duration;

use super::selection::DriverEntry;
use super::{Gravity, RotationDriver, DEFAULT_POLL_INTERVAL};

/// Failed reads in a row before the manager probes the next candidate.
const MAX_CONSECUTIVE_FAILURES: u8 = 3;

/// Owns the active driver and swaps it when it stops producing readings.
#[derive(Debug)]
pub struct DriverManager {
    pub(super) entries: Vec<DriverEntry>,
    pub(super) active: Option<usize>,
    pub(super) driver: Option<Box<dyn RotationDriver>>,
    pub(super) failures: u8,
}

impl DriverManager {
    /// Probes candidates from `start` onwards, wrapping at the end.
    pub(super) fn activate_from(&mut self, start: usize) -> bool {
        for offset in 0..self.entries.len() {
            let index = (start + offset) % self.entries.len();
            if let Some(driver) = (self.entries[index].probe)() {
                tracing::info!(driver = driver.name(), "auto-rotate driver activated");
                self.active = Some(index);
                self.driver = Some(driver);
                self.failures = 0;
                return true;
            }
        }
        false
    }
}

impl RotationDriver for DriverManager {
    fn name(&self) -> &str {
        self.driver.as_ref().map_or("none", |driver| driver.name())
    }

    fn read(&mut self) -> Option<Gravity> {
        if self.driver.is_none() {
            // Retry probing at a bounded rate. A probe can itself perform
            // blocking D-Bus calls, so doing it on every poll is wasteful on a
            // machine that has no sensor.
            self.failures += 1;
            if self.failures < MAX_CONSECUTIVE_FAILURES {
                return None;
            }
            self.failures = 0;
            if !self.activate_from(0) {
                return None;
            }
        }
        match self.driver.as_mut().and_then(|driver| driver.read()) {
            Some(gravity) => {
                self.failures = 0;
                Some(gravity)
            }
            None => {
                self.failures += 1;
                if self.failures >= MAX_CONSECUTIVE_FAILURES {
                    self.failures = 0;
                    let next = self.active.map_or(0, |index| index + 1);
                    if !self.activate_from(next) {
                        tracing::warn!("no alternative auto-rotate driver available");
                    }
                }
                None
            }
        }
    }

    fn poll_interval(&self) -> Duration {
        self.driver
            .as_ref()
            .map_or(DEFAULT_POLL_INTERVAL, |driver| driver.poll_interval())
    }
}
