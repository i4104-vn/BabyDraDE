//! Wallpaper configuration and queries

use std::path::{Path, PathBuf};

/// Embedded default vector wallpaper (1920x1080 obsidian theme)
pub const DEFAULT_WALLPAPER_SVG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/wallpaper/default.svg"
));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WallpaperMode {
    #[default]
    Static,
    Live,
}

/// Returns the embedded default SVG wallpaper content
pub fn get_default_wallpaper_svg() -> &'static str {
    DEFAULT_WALLPAPER_SVG
}

/// Queries the active wallpaper path on disk, prioritizing user preferences,
/// system defaults, and bundled assets.
pub fn get_wallpaper() -> Option<PathBuf> {
    // 1. User custom wallpaper in ~/.config/babydra/wallpaper.*
    if let Ok(home) = std::env::var("HOME") {
        let config_dir = PathBuf::from(home).join(".config/babydra");
        for ext in &["png", "jpg", "jpeg", "svg", "webp"] {
            let p = config_dir.join(format!("wallpaper.{ext}"));
            if p.is_file() {
                return Some(p);
            }
        }
        let direct = config_dir.join("wallpaper");
        if direct.is_file() {
            return Some(direct);
        }
    }

    // 2. Installed system background (/usr/share/backgrounds/babydra/default.*)
    for ext in &["svg", "png", "jpg", "jpeg", "webp"] {
        let default_wp = PathBuf::from(format!("/usr/share/backgrounds/babydra/default.{ext}"));
        if default_wp.is_file() {
            return Some(default_wp);
        }
    }

    // 3. Asset directory relative to running binary
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            for candidate in &[
                exe_dir.join("assets/wallpaper/default.svg"),
                exe_dir.join("../assets/wallpaper/default.svg"),
                exe_dir.join("../../assets/wallpaper/default.svg"),
            ] {
                if candidate.is_file() {
                    return candidate.canonicalize().ok().or_else(|| Some(candidate.clone()));
                }
            }
        }
    }

    // 4. Current working directory relative assets
    for candidate in &[
        PathBuf::from("assets/wallpaper/default.svg"),
        PathBuf::from("BabyDraDE/assets/wallpaper/default.svg"),
    ] {
        if candidate.is_file() {
            return candidate.canonicalize().ok().or_else(|| Some(candidate.clone()));
        }
    }

    None
}

/// Sets and saves the user wallpaper to ~/.config/babydra/wallpaper.<ext>
pub fn set_wallpaper(source_path: &Path) -> Result<PathBuf, std::io::Error> {
    if !source_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Wallpaper file not found: {}", source_path.display()),
        ));
    }

    let home = std::env::var("HOME").map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "HOME environment variable not set")
    })?;
    let config_dir = PathBuf::from(home).join(".config/babydra");
    std::fs::create_dir_all(&config_dir)?;

    let ext = source_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let dest = config_dir.join(format!("wallpaper.{ext}"));

    // Remove old wallpapers with different extensions to avoid ambiguity
    for other_ext in &["png", "jpg", "jpeg", "svg", "webp"] {
        let old = config_dir.join(format!("wallpaper.{other_ext}"));
        if old.exists() && old != dest {
            let _ = std::fs::remove_file(old);
        }
    }

    std::fs::copy(source_path, &dest)?;
    Ok(dest)
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
