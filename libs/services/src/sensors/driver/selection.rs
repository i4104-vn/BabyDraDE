//! Driver selection from configuration.

use super::RotationDriver;

/// A registered way to build one driver.
#[derive(Clone, Copy, Debug)]
pub(super) struct DriverEntry {
    pub(super) name: &'static str,
    pub(super) aliases: &'static [&'static str],
    pub(super) probe: fn() -> Option<Box<dyn RotationDriver>>,
}

impl DriverEntry {
    /// Whether a configuration value selects this entry.
    pub(super) fn matches(&self, selection: &str) -> bool {
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
