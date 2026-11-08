use std::fs;
use std::path::PathBuf;

pub fn list_available_icon_themes() -> Vec<String> {
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
            // A valid icon theme directory must contain an index.theme file
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

    themes
}
