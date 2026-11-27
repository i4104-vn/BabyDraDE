//! Registry of available drivers, in probing order.

use super::selection::DriverEntry;
use super::{DriverManager, DriverSelection, RotationDriver};
use crate::sensors::rotation::drivers;

/// Registry of available drivers, in probing order.
#[derive(Default)]
pub struct DriverFactory {
    entries: Vec<DriverEntry>,
}

impl DriverFactory {
    /// Builds a factory containing every driver compiled into the service.
    pub fn with_builtins() -> Self {
        Self::default()
            .register(
                "gaokun3",
                &["gaokun", "linux-gaokun", "ssc", "slpi", "matebook-ego"],
                drivers::gaokun3::Gaokun3Driver::probe,
            )
            .register(
                "sensor-proxy",
                &["proxy", "dbus", "iio-sensor-proxy"],
                drivers::sensor_proxy::SensorProxyDriver::probe,
            )
            .register(
                "iio",
                &["sysfs", "accelerometer"],
                drivers::iio::IioDriver::probe,
            )
            .register(
                "evdev",
                &["input", "event"],
                drivers::evdev::EvdevDriver::probe,
            )
    }

    /// Adds a driver candidate. Later registrations are probed after earlier ones.
    ///
    /// # Arguments
    /// * `name` - Canonical configuration value.
    /// * `aliases` - Accepted alternative names.
    /// * `probe` - Constructor returning `None` when unavailable on this machine.
    pub fn register(
        mut self,
        name: &'static str,
        aliases: &'static [&'static str],
        probe: fn() -> Option<Box<dyn RotationDriver>>,
    ) -> Self {
        self.entries.push(DriverEntry {
            name,
            aliases,
            probe,
        });
        self
    }

    /// Resolves a selection into a manager that can activate its candidates.
    ///
    /// An unknown name warns and falls back to auto detection, so a config typo
    /// does not disable rotation on a machine where a sensor exists.
    pub fn create(&self, selection: DriverSelection) -> DriverManager {
        let entries = match selection {
            DriverSelection::Auto => self.entries.clone(),
            DriverSelection::Named(name) => {
                let matched: Vec<_> = self
                    .entries
                    .iter()
                    .filter(|entry| entry.matches(&name))
                    .copied()
                    .collect();
                if matched.is_empty() {
                    tracing::warn!(driver = %name, "unknown auto-rotate driver; probing all");
                    self.entries.clone()
                } else {
                    matched
                }
            }
        };
        let mut manager = DriverManager {
            entries,
            active: None,
            driver: None,
            failures: 0,
        };
        // Probe at startup so callers can tell immediately whether this
        // machine has a usable sensor.
        manager.activate_from(0);
        manager
    }
}
