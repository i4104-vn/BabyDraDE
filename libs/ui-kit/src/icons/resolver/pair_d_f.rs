//! Icon asset pairs for names starting with d_f.

pub(super) fn resolve(name: &str) -> Option<(&'static str, &'static str)> {
    match name {
        "dark-mode" => Some((
            crate::icons::assets::DARK_DARK_MODE_SVG,
            crate::icons::assets::LIGHT_DARK_MODE_SVG,
        )),
        "display" => Some((
            crate::icons::assets::DARK_DISPLAY_SVG,
            crate::icons::assets::LIGHT_DISPLAY_SVG,
        )),
        "down" => Some((crate::icons::assets::DARK_DOWN_SVG, crate::icons::assets::LIGHT_DOWN_SVG)),
        "download" => Some((
            crate::icons::assets::DARK_DOWNLOAD_SVG,
            crate::icons::assets::LIGHT_DOWNLOAD_SVG,
        )),
        "drive-harddisk" => Some((
            crate::icons::assets::DARK_DRIVE_HARDDISK_SVG,
            crate::icons::assets::LIGHT_DRIVE_HARDDISK_SVG,
        )),
        "edit" => Some((crate::icons::assets::DARK_EDIT_SVG, crate::icons::assets::LIGHT_EDIT_SVG)),
        "ellipse" => Some((
            crate::icons::assets::DARK_ELLIPSE_SVG,
            crate::icons::assets::LIGHT_ELLIPSE_SVG,
        )),
        "eraser" => Some((
            crate::icons::assets::DARK_ERASER_SVG,
            crate::icons::assets::LIGHT_ERASER_SVG,
        )),
        "ethernet" => Some((
            crate::icons::assets::DARK_ETHERNET_SVG,
            crate::icons::assets::LIGHT_ETHERNET_SVG,
        )),
        "external-link" => Some((
            crate::icons::assets::DARK_EXTERNAL_LINK_SVG,
            crate::icons::assets::LIGHT_EXTERNAL_LINK_SVG,
        )),
        "eye" => Some((crate::icons::assets::DARK_EYE_SVG, crate::icons::assets::LIGHT_EYE_SVG)),
        "eye-off" => Some((
            crate::icons::assets::DARK_EYE_OFF_SVG,
            crate::icons::assets::LIGHT_EYE_OFF_SVG,
        )),
        "folder" => Some((
            crate::icons::assets::DARK_FOLDER_SVG,
            crate::icons::assets::LIGHT_FOLDER_SVG,
        )),
        "folder-desktop" => Some((
            crate::icons::assets::DARK_FOLDER_DESKTOP_SVG,
            crate::icons::assets::LIGHT_FOLDER_DESKTOP_SVG,
        )),
        "folder-documents" => Some((
            crate::icons::assets::DARK_FOLDER_DOCUMENTS_SVG,
            crate::icons::assets::LIGHT_FOLDER_DOCUMENTS_SVG,
        )),
        "folder-download" => Some((
            crate::icons::assets::DARK_FOLDER_DOWNLOAD_SVG,
            crate::icons::assets::LIGHT_FOLDER_DOWNLOAD_SVG,
        )),
        "folder-music" => Some((
            crate::icons::assets::DARK_FOLDER_MUSIC_SVG,
            crate::icons::assets::LIGHT_FOLDER_MUSIC_SVG,
        )),
        "folder-new" => Some((
            crate::icons::assets::DARK_FOLDER_NEW_SVG,
            crate::icons::assets::LIGHT_FOLDER_NEW_SVG,
        )),
        "folder-pictures" => Some((
            crate::icons::assets::DARK_FOLDER_PICTURES_SVG,
            crate::icons::assets::LIGHT_FOLDER_PICTURES_SVG,
        )),
        "folder-videos" => Some((
            crate::icons::assets::DARK_FOLDER_VIDEOS_SVG,
            crate::icons::assets::LIGHT_FOLDER_VIDEOS_SVG,
        )),
        "forward" => Some((
            crate::icons::assets::DARK_FORWARD_SVG,
            crate::icons::assets::LIGHT_FORWARD_SVG,
        )),
        "env" => Some((
            crate::icons::assets::DARK_SETTINGS_SVG,
            crate::icons::assets::LIGHT_SETTINGS_SVG,
        )),
        _ => None,
    }
}
