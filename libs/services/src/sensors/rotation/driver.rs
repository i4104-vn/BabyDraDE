//! Pluggable accelerometer drivers and the factory that selects them.
//!
//! A driver only knows how to produce a gravity sample. The factory owns an
//! ordered list of candidates, supports explicit selection from configuration,
//! and the manager replaces a driver that stops producing readings. Adding a
//! driver for a new machine means registering one probe function; no consumer
//! code changes.

use std::time::Duration;

use super::{drivers, Gravity};

/// Sampling interval used when a driver does not request its own.
pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(500);
/// Failed reads in a row before the manager probes the next candidate.
const MAX_CONSECUTIVE_FAILURES: u8 = 3;

/// A source of gravity samples.
pub trait RotationDriver {
    /// Identifier used in logs and configuration (`iio`, `sensor-proxy`, …).
    fn name(&self) -> &str;

    /// Reads the current acceleration; `None` means a transient failure.
    fn read(&mut self) -> Option<Gravity>;

    /// Sampling interval preferred by this driver.
    fn poll_interval(&self) -> Duration {
        DEFAULT_POLL_INTERVAL
    }
}

/// A registered way to build one driver.
#[derive(Clone, Copy)]
struct DriverEntry {
    name: &'static str,
    aliases: &'static [&'static str],
    probe: fn() -> Option<Box<dyn RotationDriver>>,
}

impl DriverEntry {
    /// Whether a configuration value selects this entry.
    fn matches(&self, selection: &str) -> bool {
        let value = selection.trim().to_ascii_lowercase();
        self.name == value || self.aliases.iter().any(|alias| *alias == value)
    }
}

/// Driver chosen by configuration, or automatic detection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DriverSelection {
    Auto,
    Named(String),
}

impl DriverSelection {
    /// Parses the `auto_rotate_driver` configuration value.
    ///
    /// # Arguments
    /// * `value` - Configured driver name, if any.
    ///
    /// # Returns
    /// `Auto` when unset, empty or set to `auto`; otherwise the named driver.
    pub fn from_config(value: Option<&str>) -> Self {
        match value.map(str::trim).filter(|value| !value.is_empty()) {
            None => Self::Auto,
            Some("auto") => Self::Auto,
            Some(name) => Self::Named(name.to_ascii_lowercase()),
        }
    }
}

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
                &["ssc", "slpi", "matebook-ego"],
                drivers::gaokun3::Gaokun3Driver::probe,
            )
            .register(
                "iio",
                &["sysfs", "accelerometer"],
                drivers::iio::IioDriver::probe,
            )
            .register(
                "sensor-proxy",
                &["proxy", "dbus", "iio-sensor-proxy"],
                drivers::sensor_proxy::SensorProxyDriver::probe,
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

/// Owns the active driver and swaps it when it stops producing readings.
pub struct DriverManager {
    entries: Vec<DriverEntry>,
    active: Option<usize>,
    driver: Option<Box<dyn RotationDriver>>,
    failures: u8,
}

impl DriverManager {
    /// Probes candidates from `start` onwards, wrapping at the end.
    fn activate_from(&mut self, start: usize) -> bool {
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
        if self.driver.is_none() && !self.activate_from(0) {
            return None;
        }
        match self.driver.as_mut().and_then(|driver| driver.read()) {
            Some(gravity) => {
                self.failures = 0;
                Some(gravity)
            }
            None => {
                self.failures += 1;
                if self.failures >= MAX_CONSECUTIVE_FAILURES {
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

#[cfg(test)]
mod tests {
    use super::{DriverFactory, DriverSelection, Gravity, RotationDriver};

    struct FakeDriver {
        name: &'static str,
        value: Option<Gravity>,
    }

    impl RotationDriver for FakeDriver {
        fn name(&self) -> &str {
            self.name
        }
        fn read(&mut self) -> Option<Gravity> {
            self.value
        }
    }

    fn down() -> Option<Box<dyn RotationDriver>> {
        Some(Box::new(FakeDriver {
            name: "iio",
            value: Some(Gravity { x: 0.0, y: 1.0 }),
        }))
    }

    fn unavailable() -> Option<Box<dyn RotationDriver>> {
        None
    }

    fn proxy() -> Option<Box<dyn RotationDriver>> {
        Some(Box::new(FakeDriver {
            name: "sensor-proxy",
            value: Some(Gravity { x: 1.0, y: 0.0 }),
        }))
    }

    fn failing() -> Option<Box<dyn RotationDriver>> {
        Some(Box::new(FakeDriver {
            name: "iio",
            value: None,
        }))
    }

    #[test]
    fn auto_selection_falls_back_when_primary_probe_fails() {
        let factory = DriverFactory::default()
            .register("iio", &[], unavailable)
            .register("sensor-proxy", &[], proxy);
        let manager = factory.create(DriverSelection::Auto);
        assert_eq!(manager.name(), "sensor-proxy");
    }

    #[test]
    fn named_selection_uses_only_the_matching_driver() {
        let factory = DriverFactory::default()
            .register("iio", &[], down)
            .register("sensor-proxy", &["proxy"], proxy);
        let manager = factory.create(DriverSelection::Named("PROXY".into()));
        assert_eq!(manager.name(), "sensor-proxy");
    }

    #[test]
    fn unknown_selection_falls_back_to_auto_detection() {
        let factory = DriverFactory::default()
            .register("iio", &[], down)
            .register("sensor-proxy", &[], proxy);
        let manager = factory.create(DriverSelection::Named("bogus".into()));
        assert_eq!(manager.name(), "iio");
    }

    #[test]
    fn switches_driver_after_repeated_read_failures() {
        let factory = DriverFactory::default()
            .register("iio", &[], failing)
            .register("sensor-proxy", &[], proxy);
        let mut manager = factory.create(DriverSelection::Auto);
        for _ in 0..3 {
            let _ = manager.read();
        }
        assert_eq!(manager.name(), "sensor-proxy");
    }
}
