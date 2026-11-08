use std::fs;
use std::io;
use std::path::PathBuf;

pub fn apply_cursor_theme(cursor_theme: &str, cursor_size: u32) -> io::Result<()> {
    // 1. Set Environment Variables for all spawned processes in the desktop session
    std::env::set_var("XCURSOR_THEME", cursor_theme);
    std::env::set_var("XCURSOR_SIZE", cursor_size.to_string());

    // 2. Write ~/.icons/default/index.theme for legacy & toolkits
    if let Ok(home) = std::env::var("HOME") {
        let default_icon_dir = PathBuf::from(home).join(".icons").join("default");
        fs::create_dir_all(&default_icon_dir)?;

        let content = format!(
            r#"[Icon Theme]
Name=Default
Comment=Default Cursor Theme
Inherits={cursor_theme}
"#
        );
        fs::write(default_icon_dir.join("index.theme"), content)?;
    }

    Ok(())
}

/// Discovers installed cursor themes in standard Linux icon directories
pub fn list_available_cursor_themes() -> Vec<String> {
    let mut themes = Vec::new();
    let mut search_paths = vec![
        PathBuf::from("/usr/share/icons"),
        PathBuf::from("/usr/local/share/icons"),
    ];

    if let Ok(home) = std::env::var("HOME") {
        let home_p = PathBuf::from(home);
        search_paths.push(home_p.join(".icons"));
        search_paths.push(home_p.join(".local/share/icons"));
    }

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
