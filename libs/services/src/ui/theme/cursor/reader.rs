use std::fs;
use std::path::PathBuf;

/// Cursor size used when neither `XCURSOR_SIZE` nor the theme config supplies one.
const DEFAULT_CURSOR_SIZE: u32 = 24;

/// Resolved cursor theme name, size and directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorThemeInfo {
    pub name: String,
    pub size: u32,
    pub path: Option<PathBuf>,
}

/// Resolves the active cursor theme from environment and GTK settings.
///
/// # Returns
/// Name, size and optional path of the active cursor theme.
pub fn read_active_cursor_theme() -> CursorThemeInfo {
    let env_name = std::env::var("XCURSOR_THEME")
        .ok()
        .filter(|s| !s.is_empty());
    let env_size = std::env::var("XCURSOR_SIZE")
        .ok()
        .and_then(|s| s.parse::<u32>().ok());

    let file_theme = read_default_index_theme();

    let name = env_name
        .or(file_theme)
        .unwrap_or_else(|| "Adwaita".to_string());

    let size = env_size.unwrap_or(DEFAULT_CURSOR_SIZE);
    let path = find_cursor_theme_path(&name);

    CursorThemeInfo { name, size, path }
}

/// Reads the default cursor theme name from `index.theme`.
///
/// # Returns
/// The theme name, or `None` when unavailable.
pub fn read_default_index_theme() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let path = PathBuf::from(home)
        .join(".icons")
        .join("default")
        .join("index.theme");
    let content = fs::read_to_string(path).ok()?;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Inherits=") || trimmed.starts_with("Inherits =") {
            let parts: Vec<&str> = trimmed.split('=').collect();
            if parts.len() == 2 {
                let theme = parts[1].trim();
                if !theme.is_empty() {
                    return Some(theme.to_string());
                }
            }
        }
    }

    None
}

/// Locates the directory containing a cursor theme.
///
/// # Arguments
/// * `name` - Cursor theme name.
///
/// # Returns
/// The theme directory, or `None` if not found.
pub fn find_cursor_theme_path(name: &str) -> Option<PathBuf> {
    let search_paths = cursor_theme_search_paths();
    for base in search_paths {
        let candidate = base.join(name).join("cursors");
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    None
}

/// Lists cursor themes installed in standard search paths.
///
/// # Returns
/// Sorted, deduplicated cursor theme names.
pub fn list_available_cursor_themes() -> Vec<String> {
    let mut themes = Vec::new();
    let search_paths = cursor_theme_search_paths();

    for base in search_paths {
        let entries = match fs::read_dir(base) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.join("cursors").is_dir() {
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy().to_string();
                    if !themes.contains(&name_str) {
                        themes.push(name_str);
                    }
                }
            }
        }
    }

    themes
}

/// Directories that contain cursor theme folders (`<theme>/cursors/`).
///
/// `~/.babydra/cursor` takes priority so locally installed themes can override
/// system-wide ones with the same name.
pub(crate) fn cursor_theme_search_paths() -> Vec<PathBuf> {
    let mut paths = vec![crate::config::get_cursor_dir()];

    if let Ok(home) = std::env::var("HOME") {
        let home_p = PathBuf::from(home);
        paths.push(home_p.join(".icons"));
        paths.push(home_p.join(".local/share/icons"));
    }

    paths.push(PathBuf::from("/usr/share/icons"));
    paths.push(PathBuf::from("/usr/local/share/icons"));

    paths
}
