//! Desktop shell data and file manager backend

pub use crate::config::{load_desktop_config, save_desktop_config};
pub use crate::models::DesktopConfig;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopEntry {
    pub name: String,
    pub path: PathBuf,
    pub icon_name: String,
    pub is_dir: bool,
}

pub fn fetch_desktop_entries() -> Vec<DesktopEntry> {
    let mut entries = Vec::new();
    let desktop_dir = std::env::var("HOME")
        .map(|h| PathBuf::from(h).join("Desktop"))
        .unwrap_or_else(|_| PathBuf::from("/home"));
    let _ = std::fs::create_dir_all(&desktop_dir);

    if let Ok(dir_entries) = std::fs::read_dir(&desktop_dir) {
        for entry in dir_entries.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            let is_dir = path.is_dir();
            let icon_name = if is_dir {
                "folder".to_string()
            } else if path.extension().map(|e| e == "desktop").unwrap_or(false) {
                "application-x-executable".to_string()
            } else {
                "text-x-generic".to_string()
            };

            entries.push(DesktopEntry {
                name,
                path,
                icon_name,
                is_dir,
            });
        }
    }

    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    entries
}
