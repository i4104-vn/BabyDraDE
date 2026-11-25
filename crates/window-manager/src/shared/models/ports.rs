/// Structured failure returned when a config snapshot cannot be loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConfigError {
pub(crate) message: String,
}

/// Structured failure returned when a configured command cannot be launched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LaunchError {
pub(crate) message: String,
}
