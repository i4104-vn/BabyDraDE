use serde::{Deserialize, Serialize};

/// Represents an application entry parsed from a `.desktop` file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopEntry {
    pub id: String,
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub comment: Option<String>,
    pub categories: Vec<String>,
    pub terminal: bool,
}

impl DesktopEntry {
    pub fn new(id: impl Into<String>, name: impl Into<String>, exec: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            exec: exec.into(),
            icon: None,
            comment: None,
            categories: Vec::new(),
            terminal: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppRegistryError {
    ScanFailed(String),
    LaunchFailed(String),
}

impl std::fmt::Display for AppRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ScanFailed(msg) => write!(f, "Failed to scan application registry: {msg}"),
            Self::LaunchFailed(msg) => write!(f, "Failed to launch application: {msg}"),
        }
    }
}

impl std::error::Error for AppRegistryError {}

/// Contract for scanning, indexing and searching system applications.
pub trait AppRegistry {
    /// Scans standard desktop entry directories and refreshes index.
    fn refresh(&mut self) -> Result<Vec<DesktopEntry>, AppRegistryError>;

    /// Finds application by desktop id (e.g. "firefox.desktop").
    fn find_by_id(&self, id: &str) -> Option<DesktopEntry>;

    /// Searches indexed applications by matching query in name, exec or comment.
    fn search(&self, query: &str) -> Vec<DesktopEntry>;
}
