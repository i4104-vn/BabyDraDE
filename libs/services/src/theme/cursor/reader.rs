use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorThemeInfo {
    pub name: String,
    pub size: u32,
    pub path: Option<PathBuf>,
}

pub fn read_active_cursor_theme() -> CursorThemeInfo {
    // 1. Check environment variables
    let env_name = std::env::var("XCURSOR_THEME")
        .ok()
        .filter(|s| !s.is_empty());
    let env_size = std::env::var("XCURSOR_SIZE")
        .ok()
        .and_then(|s| s.parse::<u32>().ok());

    // 2. Fallback: ~/.icons/default/index.theme
    let file_theme = read_default_index_theme();

    let name = env_name
        .or(file_theme)
        .unwrap_or_else(|| "Adwaita".to_string());

    let size = env_size.unwrap_or(24);
    let path = find_cursor_theme_path(&name);

    CursorThemeInfo { name, size, path }
}

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

pub fn find_cursor_theme_path(name: &str) -> Option<PathBuf> {
    let search_paths = get_icon_search_paths();
    for base in search_paths {
        let candidate = base.join(name).join("cursors");
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    None
}

pub fn list_available_cursor_themes() -> Vec<String> {
    let mut themes = Vec::new();
    let search_paths = get_icon_search_paths();

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

fn get_icon_search_paths() -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from("/usr/share/icons"),
        PathBuf::from("/usr/local/share/icons"),
    ];

    if let Ok(home) = std::env::var("HOME") {
        let home_p = PathBuf::from(home);
        paths.push(home_p.join(".icons"));
        paths.push(home_p.join(".local/share/icons"));
    }

    paths
}
