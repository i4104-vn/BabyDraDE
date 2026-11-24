use std::fs;
use std::path::PathBuf;

/// Locates the user GTK4 CSS file.
///
/// # Returns
/// The path to `~/.config/gtk-4.0/gtk.css`, if it exists.
pub fn find_gtk4_css_path() -> Option<PathBuf> {
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(xdg_config).join("gtk-4.0").join("gtk.css");
        if p.is_file() {
            return Some(p);
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home)
            .join(".config")
            .join("gtk-4.0")
            .join("gtk.css");
        if p.is_file() {
            return Some(p);
        }
    }

    None
}

/// Reads user GTK4 CSS overrides.
///
/// # Returns
/// The CSS contents, or `None` if the file cannot be read.
pub fn read_gtk4_css() -> Option<String> {
    let path = find_gtk4_css_path()?;
    fs::read_to_string(path).ok()
}
