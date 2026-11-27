use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use url::Url;

use super::launch::{fallback_open_file, launch, launch_uri};

pub(super) const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "markdown", "rs", "c", "h", "cpp", "hpp", "cxx", "py", "js", "ts", "jsx", "tsx",
    "json", "toml", "yaml", "yml", "sh", "bash", "zsh", "css", "scss", "html", "xml", "csv", "ini",
    "conf", "log", "diff", "patch", "desktop", "service", "sql", "lua", "go", "java",
];
pub(super) const MEDIA_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "gif", "bmp", "svg", "ico", "mp4", "mkv", "webm", "avi", "mov",
    "flv",
];

/// Opens a path or URI with the desktop's default application; returns `true` on success.
pub fn open_target(target: &str) -> bool {
    if ["http://", "https://", "mailto:", "ftp://"]
        .into_iter()
        .any(|s| target.starts_with(s))
    {
        if launch_uri(target) {
            return true;
        }
        for browser in [
            "opera",
            "google-chrome-stable",
            "google-chrome",
            "firefox",
            "chromium",
        ] {
            if launch(browser, &[OsStr::new(target)]) {
                return true;
            }
        }
        tracing::warn!("no handler for URI {target}");
        return false;
    }
    let path = resolve_path(target);
    if path.is_dir() {
        return show_in_file_manager(&path);
    }
    let uri = file_uri(&path);
    launch_uri(&uri) || fallback_open_file(&path, &uri)
}

/// Opens a directory in BabyDra Explore, falling back to GIO.
fn show_in_file_manager(path: &Path) -> bool {
    launch("babydra-explore", &[path.as_os_str()]) || launch_uri(&file_uri(path))
}

/// Resolves a possibly relative path or `file://` URI into an absolute path.
fn resolve_path(target: &str) -> PathBuf {
    let raw = Url::parse(target)
        .ok()
        .filter(|u| u.scheme() == "file")
        .and_then(|u| u.to_file_path().ok())
        .unwrap_or_else(|| PathBuf::from(target));
    let absolute = if raw.is_absolute() {
        raw
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(&raw))
            .unwrap_or(raw)
    };
    std::fs::canonicalize(&absolute).unwrap_or(absolute)
}

/// Builds a `file://` URI for a local path.
fn file_uri(path: &Path) -> String {
    Url::from_file_path(path)
        .map(|u| u.to_string())
        .unwrap_or_else(|_| format!("file://{}", path.to_string_lossy()))
}
