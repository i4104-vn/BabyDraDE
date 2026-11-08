use std::fs;
use std::io;

use super::config::get_config_dir;

pub fn apply_gtk4_css(css: &str) -> io::Result<()> {
    let gtk4_dir = get_config_dir().join("gtk-4.0");
    fs::create_dir_all(&gtk4_dir)?;
    fs::write(gtk4_dir.join("gtk.css"), css)
}

pub fn remove_gtk4_css() -> io::Result<()> {
    let css_file = get_config_dir().join("gtk-4.0").join("gtk.css");
    if css_file.exists() {
        fs::remove_file(css_file)?;
    }
    Ok(())
}

pub fn read_current_gtk4_css() -> Option<String> {
    let css_file = get_config_dir().join("gtk-4.0").join("gtk.css");
    fs::read_to_string(css_file).ok()
}
