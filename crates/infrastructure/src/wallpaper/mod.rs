//! Wallpaper configuration and queries matching babydra-core

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

impl WallpaperMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Live => "live",
        }
    }
}

/// Returns the embedded default SVG wallpaper content
pub fn get_default_wallpaper_svg() -> &'static str {
    DEFAULT_WALLPAPER_SVG
}

/// Returns the user's wallpaper directory: `~/.babydra/wallpaper`.
pub fn get_wallpaper_dir() -> PathBuf {
    crate::config::get_wallpaper_dir()
}

/// Queries the active wallpaper path on disk, prioritizing `~/.babydra/babydra.conf`,
/// `~/.babydra/wallpaper/`, legacy paths, and bundled assets.
pub fn get_wallpaper() -> Option<PathBuf> {
    crate::config::invalidate_cache();
    let conf = crate::config::load_babydra_config();

    // 1. Current wallpaper defined in ~/.babydra/babydra.conf
    if !conf.wallpaper.current.is_empty() {
        let path = PathBuf::from(&conf.wallpaper.current);
        if path.exists() {
            return Some(path);
        }
    }

    // 2. Any image in ~/.babydra/wallpaper/
    let wp_dir = get_wallpaper_dir();
    if let Ok(entries) = std::fs::read_dir(&wp_dir) {
        let mut files: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| {
                if !p.is_file() {
                    return false;
                }
                let ext = p
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                matches!(
                    ext.as_str(),
                    "svg" | "png" | "jpg" | "jpeg" | "webp" | "mp4" | "webm" | "gif"
                )
            })
            .collect();
        files.sort();
        if let Some(first) = files.first() {
            return Some(first.clone());
        }
    }

    // 3. Default vector wallpaper in ~/.babydra/wallpaper/default.svg
    let default_wp = wp_dir.join("default.svg");
    if default_wp.is_file() {
        return Some(default_wp);
    }

    // 4. Legacy ~/.config/babydra fallback
    if let Ok(home) = std::env::var("HOME") {
        let legacy_dir = PathBuf::from(home).join(".config/babydra");
        for ext in &["png", "jpg", "jpeg", "svg", "webp"] {
            let p = legacy_dir.join(format!("wallpaper.{ext}"));
            if p.is_file() {
                return Some(p);
            }
        }
    }

    // 5. Installed system background (/usr/share/backgrounds/babydra/default.*)
    for ext in &["svg", "png", "jpg", "jpeg", "webp"] {
        let system_wp = PathBuf::from(format!("/usr/share/backgrounds/babydra/default.{ext}"));
        if system_wp.is_file() {
            return Some(system_wp);
        }
    }

    // 6. Current working directory or repo relative assets
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

/// Sets and saves the user wallpaper into `~/.babydra/wallpaper/<filename>`
/// and updates `~/.babydra/babydra.conf`.
pub fn set_wallpaper(source_path: &Path) -> Result<PathBuf, std::io::Error> {
    if !source_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Wallpaper file not found: {}", source_path.display()),
        ));
    }

    let target_dir = get_wallpaper_dir();
    std::fs::create_dir_all(&target_dir)?;

    let target_path = if source_path.parent() != Some(&target_dir) {
        if let Some(file_name) = source_path.file_name() {
            let dest = target_dir.join(file_name);
            if source_path != dest {
                std::fs::copy(source_path, &dest)?;
            }
            dest
        } else {
            source_path.to_path_buf()
        }
    } else {
        source_path.to_path_buf()
    };

    let ext = target_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mode = if matches!(ext.as_str(), "mp4" | "webm" | "mkv" | "gif") {
        "live"
    } else {
        "static"
    };

    let mut conf = crate::config::load_babydra_config();
    conf.wallpaper.current = target_path.to_string_lossy().to_string();
    conf.wallpaper.mode = mode.to_string();
    crate::config::save_babydra_config(&conf);

    Ok(target_path)
}
