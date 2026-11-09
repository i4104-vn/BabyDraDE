//! Wallpaper configuration and queries

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WallpaperMode {
    #[default]
    Static,
    Live,
}

pub fn get_wallpaper() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home).join(".config/babydra/wallpaper.png");
        if p.exists() {
            return Some(p);
        }
    }
    let default_wp = PathBuf::from("/usr/share/backgrounds/babydra/default.png");
    if default_wp.exists() {
        return Some(default_wp);
    }
    None
}

pub fn get_wallpaper_mode() -> WallpaperMode {
    WallpaperMode::Static
}

pub fn is_video_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    matches!(ext.to_lowercase().as_str(), "mp4" | "webm" | "mkv" | "avi" | "mov")
}

pub fn is_gif_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    ext.eq_ignore_ascii_case("gif")
}
