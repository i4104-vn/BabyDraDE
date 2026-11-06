use std::collections::HashMap;
use std::path::{Path, PathBuf};

use domain::{AppRegistry, AppRegistryError, DesktopEntry};

/// Freedesktop `.desktop` application scanner and parser.
#[derive(Debug, Clone)]
pub struct DesktopFileScanner {
    pub search_paths: Vec<PathBuf>,
    pub entries: HashMap<String, DesktopEntry>,
}

impl DesktopFileScanner {
    pub fn new() -> Self {
        let mut paths = Vec::new();

        // Standard Linux XDG application directories
        paths.push(PathBuf::from("/usr/share/applications"));
        paths.push(PathBuf::from("/usr/local/share/applications"));

        if let Ok(home) = std::env::var("HOME") {
            paths.push(PathBuf::from(home).join(".local/share/applications"));
        }

        let mut scanner = Self {
            search_paths: paths,
            entries: HashMap::new(),
        };

        // Add standard fallback apps so compositor has default apps ready out of the box
        scanner.register_default_fallbacks();
        scanner
    }

    pub fn with_custom_paths(paths: Vec<PathBuf>) -> Self {
        let mut scanner = Self {
            search_paths: paths,
            entries: HashMap::new(),
        };
        scanner.register_default_fallbacks();
        scanner
    }

    fn register_default_fallbacks(&mut self) {
        let default_apps = [
            ("alacritty.desktop", "Alacritty", "alacritty", "Alacritty Terminal Emulator"),
            ("firefox.desktop", "Firefox", "firefox", "Mozilla Firefox Web Browser"),
            ("thunar.desktop", "File Manager", "thunar", "Browse the file system"),
        ];

        for (id, name, exec, comment) in default_apps {
            let mut entry = DesktopEntry::new(id, name, exec);
            entry.comment = Some(comment.into());
            self.entries.insert(id.into(), entry);
        }
    }

    /// Parses freedesktop `.desktop` file format.
    pub fn parse_desktop_file(content: &str, file_name: &str) -> Option<DesktopEntry> {
        let mut in_desktop_entry = false;
        let mut name = None;
        let mut exec = None;
        let mut icon = None;
        let mut comment = None;
        let mut categories = Vec::new();
        let mut terminal = false;
        let mut no_display = false;
        let mut is_application = true;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                in_desktop_entry = line == "[Desktop Entry]";
                continue;
            }

            if !in_desktop_entry {
                continue;
            }

            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim();
                let val = val.trim();

                match key {
                    "Type" => is_application = val.eq_ignore_ascii_case("Application"),
                    "Name" => {
                        if name.is_none() {
                            name = Some(val.to_string());
                        }
                    }
                    "Exec" => exec = Some(val.to_string()),
                    "Icon" => icon = Some(val.to_string()),
                    "Comment" => {
                        if comment.is_none() {
                            comment = Some(val.to_string());
                        }
                    }
                    "Categories" => {
                        categories = val
                            .split(';')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                    "Terminal" => terminal = val.eq_ignore_ascii_case("true"),
                    "NoDisplay" => no_display = val.eq_ignore_ascii_case("true"),
                    _ => {}
                }
            }
        }

        if !is_application || no_display {
            return None;
        }

        let name = name?;
        let exec = exec?;
        let id = file_name.to_string();

        let mut entry = DesktopEntry::new(id, name, exec);
        entry.icon = icon;
        entry.comment = comment;
        entry.categories = categories;
        entry.terminal = terminal;

        Some(entry)
    }

    /// Scans a single directory for .desktop files.
    pub fn scan_directory(&mut self, dir: &Path) {
        if !dir.is_dir() {
            return;
        }

        let read_dir = match std::fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(_) => return,
        };

        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("desktop") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("app.desktop");
                    if let Some(desktop_entry) = Self::parse_desktop_file(&content, file_name) {
                        self.entries.insert(desktop_entry.id.clone(), desktop_entry);
                    }
                }
            }
        }
    }
}

impl Default for DesktopFileScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl AppRegistry for DesktopFileScanner {
    fn refresh(&mut self) -> Result<Vec<DesktopEntry>, AppRegistryError> {
        let paths = self.search_paths.clone();
        for path in &paths {
            self.scan_directory(path);
        }
        Ok(self.entries.values().cloned().collect())
    }

    fn find_by_id(&self, id: &str) -> Option<DesktopEntry> {
        self.entries.get(id).cloned()
    }

    fn search(&self, query: &str) -> Vec<DesktopEntry> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            let mut all: Vec<_> = self.entries.values().cloned().collect();
            all.sort_by(|a, b| a.name.cmp(&b.name));
            return all;
        }

        let mut matched: Vec<_> = self
            .entries
            .values()
            .filter(|e| {
                e.name.to_lowercase().contains(&q)
                    || e.id.to_lowercase().contains(&q)
                    || e.exec.to_lowercase().contains(&q)
                    || e.comment
                        .as_ref()
                        .map(|c| c.to_lowercase().contains(&q))
                        .unwrap_or(false)
            })
            .cloned()
            .collect();

        matched.sort_by(|a, b| a.name.cmp(&b.name));
        matched
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_desktop_file() {
        let content = r#"
[Desktop Entry]
Version=1.0
Type=Application
Name=Alacritty
Comment=A fast, cross-platform, OpenGL terminal emulator
Exec=alacritty
Icon=Alacritty
Categories=System;TerminalEmulator;
Terminal=false
"#;
        let entry = DesktopFileScanner::parse_desktop_file(content, "alacritty.desktop").unwrap();
        assert_eq!(entry.id, "alacritty.desktop");
        assert_eq!(entry.name, "Alacritty");
        assert_eq!(entry.exec, "alacritty");
        assert_eq!(entry.icon, Some("Alacritty".into()));
        assert_eq!(
            entry.categories,
            vec!["System".to_string(), "TerminalEmulator".to_string()]
        );
        assert!(!entry.terminal);
    }

    #[test]
    fn test_parse_nodisplay_ignored() {
        let content = r#"
[Desktop Entry]
Type=Application
Name=HiddenApp
Exec=hidden
NoDisplay=true
"#;
        assert!(DesktopFileScanner::parse_desktop_file(content, "hidden.desktop").is_none());
    }

    #[test]
    fn test_search_applications() {
        let scanner = DesktopFileScanner::new();
        let results = scanner.search("term");
        assert!(!results.is_empty());
        assert!(results.iter().any(|a| a.name == "Alacritty"));

        let results_firefox = scanner.search("firefox");
        assert_eq!(results_firefox.len(), 1);
        assert_eq!(results_firefox[0].name, "Firefox");
    }
}
