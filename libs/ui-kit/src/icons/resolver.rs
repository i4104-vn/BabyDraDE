//! Pure string resolver for BabyDra icon names.
use std::collections::HashMap;
use std::sync::OnceLock;

fn aliases() -> &'static HashMap<String, String> {
    static ALIASES: OnceLock<HashMap<String, String>> = OnceLock::new();
    ALIASES.get_or_init(|| {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/icon_aliases.json"
        )))
        .unwrap_or_default()
    })
}

pub fn resolve_icon_pair(name: &str) -> Option<(&'static str, &'static str)> {
    let canonical = aliases().get(name).map(String::as_str).unwrap_or(name);
    match canonical {
        "activity" => Some((
            super::assets::DARK_ACTIVITY_SVG,
            super::assets::LIGHT_ACTIVITY_SVG,
        )),
        "airplane" => Some((
            super::assets::DARK_AIRPLANE_SVG,
            super::assets::LIGHT_AIRPLANE_SVG,
        )),
        "arrow" => Some((
            super::assets::DARK_ARROW_SVG,
            super::assets::LIGHT_ARROW_SVG,
        )),
        "back" => Some((super::assets::DARK_BACK_SVG, super::assets::LIGHT_BACK_SVG)),
        "battery" => Some((
            super::assets::DARK_BATTERY_SVG,
            super::assets::LIGHT_BATTERY_SVG,
        )),
        "bell" => Some((super::assets::DARK_BELL_SVG, super::assets::LIGHT_BELL_SVG)),
        "bell-off" => Some((
            super::assets::DARK_BELL_OFF_SVG,
            super::assets::LIGHT_BELL_OFF_SVG,
        )),
        "bluetooth" => Some((
            super::assets::DARK_BLUETOOTH_SVG,
            super::assets::LIGHT_BLUETOOTH_SVG,
        )),
        "blur" => Some((super::assets::DARK_BLUR_SVG, super::assets::LIGHT_BLUR_SVG)),
        "brightness" => Some((
            super::assets::DARK_BRIGHTNESS_SVG,
            super::assets::LIGHT_BRIGHTNESS_SVG,
        )),
        "brightness-low" => Some((
            super::assets::DARK_BRIGHTNESS_LOW_SVG,
            super::assets::LIGHT_BRIGHTNESS_LOW_SVG,
        )),
        "brightness-medium" => Some((
            super::assets::DARK_BRIGHTNESS_MEDIUM_SVG,
            super::assets::LIGHT_BRIGHTNESS_MEDIUM_SVG,
        )),
        "broom" => Some((
            super::assets::DARK_BROOM_SVG,
            super::assets::LIGHT_BROOM_SVG,
        )),
        "caffeine" => Some((
            super::assets::DARK_CAFFEINE_SVG,
            super::assets::LIGHT_CAFFEINE_SVG,
        )),
        "calendar" => Some((
            super::assets::DARK_CALENDAR_SVG,
            super::assets::LIGHT_CALENDAR_SVG,
        )),
        "camera" => Some((
            super::assets::DARK_CAMERA_SVG,
            super::assets::LIGHT_CAMERA_SVG,
        )),
        "check" => Some((
            super::assets::DARK_CHECK_SVG,
            super::assets::LIGHT_CHECK_SVG,
        )),
        "clock" => Some((
            super::assets::DARK_CLOCK_SVG,
            super::assets::LIGHT_CLOCK_SVG,
        )),
        "close" => Some((
            super::assets::DARK_CLOSE_SVG,
            super::assets::LIGHT_CLOSE_SVG,
        )),
        "copy" => Some((super::assets::DARK_COPY_SVG, super::assets::LIGHT_COPY_SVG)),
        "cut" => Some((super::assets::DARK_CUT_SVG, super::assets::LIGHT_CUT_SVG)),
        "dark-mode" => Some((
            super::assets::DARK_DARK_MODE_SVG,
            super::assets::LIGHT_DARK_MODE_SVG,
        )),
        "display" => Some((
            super::assets::DARK_DISPLAY_SVG,
            super::assets::LIGHT_DISPLAY_SVG,
        )),
        "down" => Some((super::assets::DARK_DOWN_SVG, super::assets::LIGHT_DOWN_SVG)),
        "download" => Some((
            super::assets::DARK_DOWNLOAD_SVG,
            super::assets::LIGHT_DOWNLOAD_SVG,
        )),
        "drive-harddisk" => Some((
            super::assets::DARK_DRIVE_HARDDISK_SVG,
            super::assets::LIGHT_DRIVE_HARDDISK_SVG,
        )),
        "edit" => Some((super::assets::DARK_EDIT_SVG, super::assets::LIGHT_EDIT_SVG)),
        "ellipse" => Some((
            super::assets::DARK_ELLIPSE_SVG,
            super::assets::LIGHT_ELLIPSE_SVG,
        )),
        "eraser" => Some((
            super::assets::DARK_ERASER_SVG,
            super::assets::LIGHT_ERASER_SVG,
        )),
        "ethernet" => Some((
            super::assets::DARK_ETHERNET_SVG,
            super::assets::LIGHT_ETHERNET_SVG,
        )),
        "external-link" => Some((
            super::assets::DARK_EXTERNAL_LINK_SVG,
            super::assets::LIGHT_EXTERNAL_LINK_SVG,
        )),
        "eye" => Some((super::assets::DARK_EYE_SVG, super::assets::LIGHT_EYE_SVG)),
        "eye-off" => Some((
            super::assets::DARK_EYE_OFF_SVG,
            super::assets::LIGHT_EYE_OFF_SVG,
        )),
        "folder" => Some((
            super::assets::DARK_FOLDER_SVG,
            super::assets::LIGHT_FOLDER_SVG,
        )),
        "folder-desktop" => Some((
            super::assets::DARK_FOLDER_DESKTOP_SVG,
            super::assets::LIGHT_FOLDER_DESKTOP_SVG,
        )),
        "folder-documents" => Some((
            super::assets::DARK_FOLDER_DOCUMENTS_SVG,
            super::assets::LIGHT_FOLDER_DOCUMENTS_SVG,
        )),
        "folder-download" => Some((
            super::assets::DARK_FOLDER_DOWNLOAD_SVG,
            super::assets::LIGHT_FOLDER_DOWNLOAD_SVG,
        )),
        "folder-music" => Some((
            super::assets::DARK_FOLDER_MUSIC_SVG,
            super::assets::LIGHT_FOLDER_MUSIC_SVG,
        )),
        "folder-new" => Some((
            super::assets::DARK_FOLDER_NEW_SVG,
            super::assets::LIGHT_FOLDER_NEW_SVG,
        )),
        "folder-pictures" => Some((
            super::assets::DARK_FOLDER_PICTURES_SVG,
            super::assets::LIGHT_FOLDER_PICTURES_SVG,
        )),
        "folder-videos" => Some((
            super::assets::DARK_FOLDER_VIDEOS_SVG,
            super::assets::LIGHT_FOLDER_VIDEOS_SVG,
        )),
        "forward" => Some((
            super::assets::DARK_FORWARD_SVG,
            super::assets::LIGHT_FORWARD_SVG,
        )),
        "gsconnect" => Some((
            super::assets::DARK_GSCONNECT_SVG,
            super::assets::LIGHT_GSCONNECT_SVG,
        )),
        "info" => Some((super::assets::DARK_INFO_SVG, super::assets::LIGHT_INFO_SVG)),
        "line" => Some((super::assets::DARK_LINE_SVG, super::assets::LIGHT_LINE_SVG)),
        "lock" => Some((super::assets::DARK_LOCK_SVG, super::assets::LIGHT_LOCK_SVG)),
        "logo" => Some((super::assets::DARK_LOGO_SVG, super::assets::LIGHT_LOGO_SVG)),
        "logout" => Some((
            super::assets::DARK_LOGOUT_SVG,
            super::assets::LIGHT_LOGOUT_SVG,
        )),
        "microphone" => Some((
            super::assets::DARK_MICROPHONE_SVG,
            super::assets::LIGHT_MICROPHONE_SVG,
        )),
        "minus" => Some((
            super::assets::DARK_MINUS_SVG,
            super::assets::LIGHT_MINUS_SVG,
        )),
        "music" => Some((
            super::assets::DARK_MUSIC_SVG,
            super::assets::LIGHT_MUSIC_SVG,
        )),
        "next" => Some((super::assets::DARK_NEXT_SVG, super::assets::LIGHT_NEXT_SVG)),
        "night-light" => Some((
            super::assets::DARK_NIGHT_LIGHT_SVG,
            super::assets::LIGHT_NIGHT_LIGHT_SVG,
        )),
        "paste" => Some((
            super::assets::DARK_PASTE_SVG,
            super::assets::LIGHT_PASTE_SVG,
        )),
        "pause" => Some((
            super::assets::DARK_PAUSE_SVG,
            super::assets::LIGHT_PAUSE_SVG,
        )),
        "performance" => Some((
            super::assets::DARK_PERFORMANCE_SVG,
            super::assets::LIGHT_PERFORMANCE_SVG,
        )),
        "play" => Some((super::assets::DARK_PLAY_SVG, super::assets::LIGHT_PLAY_SVG)),
        "plus" => Some((super::assets::DARK_PLUS_SVG, super::assets::LIGHT_PLUS_SVG)),
        "power" => Some((
            super::assets::DARK_POWER_SVG,
            super::assets::LIGHT_POWER_SVG,
        )),
        "previous" => Some((
            super::assets::DARK_PREVIOUS_SVG,
            super::assets::LIGHT_PREVIOUS_SVG,
        )),
        "privacy" => Some((
            super::assets::DARK_PRIVACY_SVG,
            super::assets::LIGHT_PRIVACY_SVG,
        )),
        "rect" => Some((super::assets::DARK_RECT_SVG, super::assets::LIGHT_RECT_SVG)),
        "refresh" => Some((
            super::assets::DARK_REFRESH_SVG,
            super::assets::LIGHT_REFRESH_SVG,
        )),
        "rename" => Some((
            super::assets::DARK_RENAME_SVG,
            super::assets::LIGHT_RENAME_SVG,
        )),
        "restart" => Some((
            super::assets::DARK_RESTART_SVG,
            super::assets::LIGHT_RESTART_SVG,
        )),
        "search" => Some((
            super::assets::DARK_SEARCH_SVG,
            super::assets::LIGHT_SEARCH_SVG,
        )),
        "server" => Some((
            super::assets::DARK_SERVER_SVG,
            super::assets::LIGHT_SERVER_SVG,
        )),
        "settings" => Some((
            super::assets::DARK_SETTINGS_SVG,
            super::assets::LIGHT_SETTINGS_SVG,
        )),
        "shield" => Some((
            super::assets::DARK_SHIELD_SVG,
            super::assets::LIGHT_SHIELD_SVG,
        )),
        "sidebar" => Some((
            super::assets::DARK_SIDEBAR_SVG,
            super::assets::LIGHT_SIDEBAR_SVG,
        )),
        "sleep" => Some((
            super::assets::DARK_SLEEP_SVG,
            super::assets::LIGHT_SLEEP_SVG,
        )),
        "stop" => Some((super::assets::DARK_STOP_SVG, super::assets::LIGHT_STOP_SVG)),
        "terminal" => Some((
            super::assets::DARK_TERMINAL_SVG,
            super::assets::LIGHT_TERMINAL_SVG,
        )),
        "text" => Some((super::assets::DARK_TEXT_SVG, super::assets::LIGHT_TEXT_SVG)),
        "trash" => Some((
            super::assets::DARK_TRASH_SVG,
            super::assets::LIGHT_TRASH_SVG,
        )),
        "unlock" => Some((
            super::assets::DARK_UNLOCK_SVG,
            super::assets::LIGHT_UNLOCK_SVG,
        )),
        "up" => Some((super::assets::DARK_UP_SVG, super::assets::LIGHT_UP_SVG)),
        "user" => Some((super::assets::DARK_USER_SVG, super::assets::LIGHT_USER_SVG)),
        "user-home" => Some((
            super::assets::DARK_USER_HOME_SVG,
            super::assets::LIGHT_USER_HOME_SVG,
        )),
        "user-trash" => Some((
            super::assets::DARK_USER_TRASH_SVG,
            super::assets::LIGHT_USER_TRASH_SVG,
        )),
        "view-grid" => Some((
            super::assets::DARK_VIEW_GRID_SVG,
            super::assets::LIGHT_VIEW_GRID_SVG,
        )),
        "view-list" => Some((
            super::assets::DARK_VIEW_LIST_SVG,
            super::assets::LIGHT_VIEW_LIST_SVG,
        )),
        "volume" => Some((
            super::assets::DARK_VOLUME_SVG,
            super::assets::LIGHT_VOLUME_SVG,
        )),
        "volume-low" => Some((
            super::assets::DARK_VOLUME_LOW_SVG,
            super::assets::LIGHT_VOLUME_LOW_SVG,
        )),
        "volume-mute" => Some((
            super::assets::DARK_VOLUME_MUTE_SVG,
            super::assets::LIGHT_VOLUME_MUTE_SVG,
        )),
        "wifi" => Some((super::assets::DARK_WIFI_SVG, super::assets::LIGHT_WIFI_SVG)),
        "zoom-fit" => Some((
            super::assets::DARK_ZOOM_FIT_SVG,
            super::assets::LIGHT_ZOOM_FIT_SVG,
        )),
        "zoom-in" => Some((
            super::assets::DARK_ZOOM_IN_SVG,
            super::assets::LIGHT_ZOOM_IN_SVG,
        )),
        "zoom-out" => Some((
            super::assets::DARK_ZOOM_OUT_SVG,
            super::assets::LIGHT_ZOOM_OUT_SVG,
        )),
        "palette" => Some((
            super::assets::DARK_DISPLAY_SVG,
            super::assets::LIGHT_DISPLAY_SVG,
        )),
        "sliders" => Some((
            super::assets::DARK_SETTINGS_SVG,
            super::assets::LIGHT_SETTINGS_SVG,
        )),
        "th-large" => Some((
            super::assets::DARK_VIEW_GRID_SVG,
            super::assets::LIGHT_VIEW_GRID_SVG,
        )),
        "cog" => Some((
            super::assets::DARK_SETTINGS_SVG,
            super::assets::LIGHT_SETTINGS_SVG,
        )),
        "history" => Some((
            super::assets::DARK_REFRESH_SVG,
            super::assets::LIGHT_REFRESH_SVG,
        )),
        "keybinds" => Some((
            super::assets::DARK_SETTINGS_SVG,
            super::assets::LIGHT_SETTINGS_SVG,
        )),
        "env" => Some((
            super::assets::DARK_SETTINGS_SVG,
            super::assets::LIGHT_SETTINGS_SVG,
        )),
        "avatar-default" => Some((super::assets::DARK_USER_SVG, super::assets::LIGHT_USER_SVG)),
        "square" => Some((super::assets::DARK_STOP_SVG, super::assets::LIGHT_STOP_SVG)),
        "circle" => Some((
            super::assets::DARK_ELLIPSE_SVG,
            super::assets::LIGHT_ELLIPSE_SVG,
        )),
        "zoom-original" => Some((
            super::assets::DARK_ZOOM_FIT_SVG,
            super::assets::LIGHT_ZOOM_FIT_SVG,
        )),
        "zoom-reset" => Some((
            super::assets::DARK_ZOOM_FIT_SVG,
            super::assets::LIGHT_ZOOM_FIT_SVG,
        )),
        _ => None,
    }
}

pub fn resolve_icon(name: &str, dark_mode: bool) -> Option<&'static str> {
    let (dark, light) = resolve_icon_pair(name)?;
    Some(if dark_mode { dark } else { light })
}
