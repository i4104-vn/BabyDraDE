//! Formatting helpers and breadcrumb decomposition for the file chooser.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Local};

/// Converts a path into ordered breadcrumb labels and targets.
pub(super) fn breadcrumb_segments(path: &Path) -> Vec<(String, PathBuf)> {
    let mut segments = Vec::new();
    let mut current = Some(path);
    while let Some(part) = current {
        if let Some(name) = part.file_name().and_then(|name| name.to_str()) {
            segments.push((name.to_string(), part.to_path_buf()));
        } else if part == Path::new("/") {
            segments.push(("/".to_string(), PathBuf::from("/")));
        }
        current = part.parent();
    }
    segments.reverse();
    segments
}

/// Maps a file extension to a thematic icon name.
pub(super) fn file_icon(name: &str) -> &'static str {
    let extension = Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "svg" | "ico" => "image-x-generic",
        "mp3" | "wav" | "flac" | "ogg" | "m4a" => "audio-x-generic",
        "mp4" | "mkv" | "webm" | "avi" | "mov" => "video-x-generic",
        "pdf" => "application-pdf",
        "zip" | "tar" | "gz" | "xz" | "7z" => "package-x-generic",
        "txt" | "md" | "rs" | "py" | "js" | "ts" | "json" | "toml" => "text-x-generic",
        _ => "text-x-preview",
    }
}

/// Formats a byte count for the size column.
pub(super) fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{size} B")
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Formats a modification timestamp for the date column.
pub(super) fn format_time(time: SystemTime) -> String {
    let datetime: DateTime<Local> = time.into();
    datetime.format("%d %b %Y, %H:%M").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_split_path_into_ordered_breadcrumbs() {
        let segments = breadcrumb_segments(Path::new("/home/user/docs"));
        let labels: Vec<&str> = segments.iter().map(|(label, _)| label.as_str()).collect();
        assert_eq!(labels, vec!["/", "home", "user", "docs"]);
    }

    #[test]
    fn should_format_human_readable_sizes() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(2048), "2.0 KB");
        assert_eq!(format_size(5 * 1024 * 1024), "5.0 MB");
    }
}
