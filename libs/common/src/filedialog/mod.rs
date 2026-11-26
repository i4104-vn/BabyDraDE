//! Serializable data contracts shared by the chooser UI and the launcher.
//!
//! This module stays presentation-agnostic; translated labels are added at the
//! call site.

use std::path::PathBuf;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Dialog operation mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum FileDialogMode {
    /// Pick a single existing file.
    #[default]
    OpenFile,
    /// Pick one or more existing files.
    OpenMultipleFiles,
    /// Pick a directory / folder.
    OpenFolder,
    /// Specify a destination file for saving.
    SaveFile,
}

/// File filter definition specifying allowed extensions and/or MIME types.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FileFilter {
    /// Human-readable description, e.g. `Images (*.png, *.jpg)`.
    pub name: String,
    /// Glob patterns such as `*.png` or `report.*`.
    pub patterns: Vec<String>,
    /// MIME types used when no glob pattern matches.
    pub mime_types: Vec<String>,
}

impl FileFilter {
    /// Builds the default filter matching every file.
    pub fn all_files() -> Self {
        Self {
            name: "All Files (*.*)".to_string(),
            patterns: vec!["*.*".to_string(), "*".to_string()],
            mime_types: Vec::new(),
        }
    }

    /// Builds a filter from a display name and extension patterns.
    ///
    /// # Arguments
    /// * `name` - Human-readable filter label.
    /// * `patterns` - Glob patterns to match.
    pub fn new(name: impl Into<String>, patterns: &[&str]) -> Self {
        Self {
            name: name.into(),
            patterns: patterns.iter().map(|pattern| pattern.to_string()).collect(),
            mime_types: Vec::new(),
        }
    }

    /// Parses a CLI filter spec such as `Text files:*.txt,*.md`.
    ///
    /// # Arguments
    /// * `spec` - `name:pattern,pattern` specification.
    ///
    /// # Returns
    /// The parsed filter, or `None` when the spec has no name or patterns.
    pub fn parse(spec: &str) -> Option<Self> {
        let (name, patterns) = spec.split_once(':')?;
        let patterns: Vec<String> = patterns
            .split(',')
            .map(|part| part.trim().to_string())
            .collect();
        if name.trim().is_empty() || patterns.iter().all(|pattern| pattern.is_empty()) {
            return None;
        }
        Some(Self::new(
            name.trim(),
            &patterns.iter().map(String::as_str).collect::<Vec<_>>(),
        ))
    }
}

/// Options configuring one file chooser invocation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FileDialogOptions {
    /// Dialog window title.
    pub title: String,
    /// Operational mode.
    pub mode: FileDialogMode,
    /// Initial folder to display.
    pub initial_folder: Option<PathBuf>,
    /// Initial filename pre-populated in save/open mode.
    pub initial_file: Option<String>,
    /// Custom label for the confirm button.
    pub accept_label: Option<String>,
    /// Available filters.
    pub filters: Vec<FileFilter>,
    /// Index of the initially selected filter.
    pub default_filter_index: Option<usize>,
    /// Whether hidden files are visible initially.
    pub show_hidden: bool,
}

impl Default for FileDialogOptions {
    fn default() -> Self {
        Self {
            title: "Open File".to_string(),
            mode: FileDialogMode::OpenFile,
            initial_folder: None,
            initial_file: None,
            accept_label: None,
            filters: vec![FileFilter::all_files()],
            default_filter_index: Some(0),
            show_hidden: false,
        }
    }
}

impl FileDialogOptions {
    /// Builds options for a directory picker.
    pub fn open_folder(initial_folder: Option<PathBuf>) -> Self {
        Self {
            title: "Open Folder".to_string(),
            mode: FileDialogMode::OpenFolder,
            initial_folder,
            accept_label: Some("Select".to_string()),
            filters: Vec::new(),
            default_filter_index: None,
            ..Self::default()
        }
    }

    /// Builds options for opening a single file.
    pub fn open_file(initial_folder: Option<PathBuf>, initial_file: Option<String>) -> Self {
        Self {
            title: "Open File".to_string(),
            mode: FileDialogMode::OpenFile,
            initial_folder,
            initial_file,
            accept_label: Some("Open".to_string()),
            ..Self::default()
        }
    }

    /// Builds options for opening several files.
    pub fn open_multiple(initial_folder: Option<PathBuf>) -> Self {
        Self {
            title: "Open Files".to_string(),
            mode: FileDialogMode::OpenMultipleFiles,
            initial_folder,
            accept_label: Some("Open".to_string()),
            ..Self::default()
        }
    }

    /// Builds options for saving a file.
    pub fn save_file(initial_folder: Option<PathBuf>, initial_name: Option<String>) -> Self {
        Self {
            title: "Save File".to_string(),
            mode: FileDialogMode::SaveFile,
            initial_folder,
            initial_file: initial_name,
            accept_label: Some("Save".to_string()),
            ..Self::default()
        }
    }
}

/// Result of one file chooser invocation.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FileDialogResult {
    /// Whether the user dismissed the dialog.
    pub cancelled: bool,
    /// Chosen paths; empty when cancelled.
    pub paths: Vec<PathBuf>,
}

impl FileDialogResult {
    /// Builds a successful result.
    pub fn success(paths: Vec<PathBuf>) -> Self {
        Self {
            cancelled: false,
            paths,
        }
    }

    /// Builds a cancellation result.
    pub fn cancel() -> Self {
        Self {
            cancelled: true,
            paths: Vec::new(),
        }
    }

    /// Returns the first chosen path, if any.
    pub fn single_path(&self) -> Option<&PathBuf> {
        if self.cancelled {
            None
        } else {
            self.paths.first()
        }
    }
}

/// Quick access bookmark shown in the chooser sidebar.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuickAccessPlace {
    /// Localized sidebar label.
    pub name: String,
    /// Directory the shortcut points at.
    pub path: PathBuf,
    /// Theme icon name.
    pub icon_name: String,
}
