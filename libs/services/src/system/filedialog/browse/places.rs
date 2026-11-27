//! Standard bookmark resolution for the file chooser sidebar.

use std::path::PathBuf;

use crate::i18n::trans;

use super::super::QuickAccessPlace;

/// Resolves user directories and system roots shown as quick access entries.
///
/// # Returns
/// Bookmarks that exist on this system, plus the filesystem root.
pub fn default_places() -> Vec<QuickAccessPlace> {
    let mut places = Vec::new();
    let Some(home) = dirs::home_dir() else {
        return vec![root_place()];
    };

    // Derive the trash path before moving `home` into the bookmark below.
    let trash = home.join(".local/share/Trash/files");
    places.push(place("file_dialog.place_home", home, "user-home"));

    let candidates = [
        (
            "file_dialog.place_desktop",
            dirs::desktop_dir(),
            "user-desktop",
        ),
        (
            "file_dialog.place_documents",
            dirs::document_dir(),
            "folder-documents",
        ),
        (
            "file_dialog.place_downloads",
            dirs::download_dir(),
            "folder-download",
        ),
        ("file_dialog.place_music", dirs::audio_dir(), "folder-music"),
        (
            "file_dialog.place_pictures",
            dirs::picture_dir(),
            "folder-pictures",
        ),
        (
            "file_dialog.place_videos",
            dirs::video_dir(),
            "folder-videos",
        ),
    ];
    for (key, path, icon) in candidates {
        if let Some(path) = path.filter(|path| path.exists()) {
            places.push(place(key, path, icon));
        }
    }

    if trash.exists() {
        places.push(place("file_dialog.place_trash", trash, "user-trash"));
    }
    places.push(root_place());
    places
}

/// Builds one translated sidebar bookmark.
fn place(key: &str, path: PathBuf, icon: &str) -> QuickAccessPlace {
    QuickAccessPlace {
        name: trans(key),
        path,
        icon_name: icon.to_string(),
    }
}

/// Returns the filesystem-root bookmark.
fn root_place() -> QuickAccessPlace {
    place(
        "file_dialog.place_root",
        PathBuf::from("/"),
        "drive-harddisk",
    )
}
