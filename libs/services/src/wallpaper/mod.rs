//! Wallpaper configuration and queries matching babydra-core

use std::path::{Path, PathBuf};

/// Embedded default wallpaper image (wallpaper.jpeg)
pub const DEFAULT_WALLPAPER_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/wallpaper/wallpaper.jpeg"
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

/// Returns the embedded default wallpaper binary content
pub fn get_default_wallpaper_bytes() -> &'static [u8] {
    DEFAULT_WALLPAPER_BYTES
}

/// Returns the user's wallpaper directory: `~/.babydra/wallpaper`.
pub fn get_wallpaper_dir() -> PathBuf {
    crate::config::get_wallpaper_dir()
}

pub fn get_wallpaper() -> Option<PathBuf> {
    crate::config::invalidate_cache();
    let conf = crate::config::load_babydra_config();

    if !conf.wallpaper.current.is_empty() {
        let path = PathBuf::from(&conf.wallpaper.current);
        if path.exists() {
            return Some(path);
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
