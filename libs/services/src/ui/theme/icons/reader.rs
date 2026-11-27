use std::fs;
use std::path::PathBuf;

/// Lists icon theme directories in standard search paths.
///
/// # Returns
/// Sorted, deduplicated icon theme names.
pub fn list_available_icon_themes() -> Vec<String> {
    let mut themes = Vec::new();
    let search_paths = get_icon_search_paths();

    for base in search_paths {
        let entries = match fs::read_dir(base) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.join("index.theme").is_file() {
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy().to_string();
                    if !themes.contains(&name_str) {
                        themes.push(name_str);
                    }
                }
            }
        }
    }

    // `read_dir` order is filesystem-dependent; the documented contract is a
    // stable, sorted list for dropdowns and tests.
    themes.sort();
    themes
}

/// Locates the directory containing an icon theme.
///
/// # Arguments
/// * `name` - Icon theme name.
///
/// # Returns
/// The theme directory, or `None` if not found.
pub fn find_icon_theme_path(name: &str) -> Option<PathBuf> {
    // Theme names come from config files; reject path separators so a crafted
    // name cannot escape the theme directories.
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
        || name.contains('\0')
    {
        return None;
    }
    let search_paths = get_icon_search_paths();
    for base in search_paths {
        let candidate = base.join(name);
        if candidate.join("index.theme").is_file() {
            return Some(candidate);
        }
    }
    None
}

/// All base directories that may contain icon themes.
pub fn get_icon_search_paths() -> Vec<PathBuf> {
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
