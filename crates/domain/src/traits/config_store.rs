#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigStoreError {
    NotFound,
    ReadFailed(String),
    WriteFailed(String),
    ParseError(String),
}

impl std::fmt::Display for ConfigStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Configuration file not found"),
            Self::ReadFailed(msg) => write!(f, "Failed to read configuration: {msg}"),
            Self::WriteFailed(msg) => write!(f, "Failed to write configuration: {msg}"),
            Self::ParseError(msg) => write!(f, "Failed to parse configuration: {msg}"),
        }
    }
}

impl std::error::Error for ConfigStoreError {}

/// Contract for persisting and retrieving user configuration (TOML).
pub trait ConfigStore {
    /// Loads raw configuration string (typically TOML format).
    fn load_raw_config(&self) -> Result<String, ConfigStoreError>;

    /// Saves raw configuration string to storage.
    fn save_raw_config(&mut self, content: &str) -> Result<(), ConfigStoreError>;
}
