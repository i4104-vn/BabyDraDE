//! Embedded SVG icon assets.

macro_rules! svg_pairs {
    ($($stem:literal => $dark:ident, $light:ident;)*) => {
        $(
            pub const $dark: &str = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/dark/",
                $stem,
                ".svg",
            ));
            pub const $light: &str = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/light/",
                $stem,
                ".svg",
            ));
        )*
    };
}

svg_pairs! {
    "activity" => DARK_ACTIVITY_SVG, LIGHT_ACTIVITY_SVG;
    "airplane" => DARK_AIRPLANE_SVG, LIGHT_AIRPLANE_SVG;
    "arrow" => DARK_ARROW_SVG, LIGHT_ARROW_SVG;
    "back" => DARK_BACK_SVG, LIGHT_BACK_SVG;
    "battery" => DARK_BATTERY_SVG, LIGHT_BATTERY_SVG;
    "bell" => DARK_BELL_SVG, LIGHT_BELL_SVG;
    "bell-off" => DARK_BELL_OFF_SVG, LIGHT_BELL_OFF_SVG;
    "bluetooth" => DARK_BLUETOOTH_SVG, LIGHT_BLUETOOTH_SVG;
    "blur" => DARK_BLUR_SVG, LIGHT_BLUR_SVG;
    "brightness" => DARK_BRIGHTNESS_SVG, LIGHT_BRIGHTNESS_SVG;
    "brightness-low" => DARK_BRIGHTNESS_LOW_SVG, LIGHT_BRIGHTNESS_LOW_SVG;
    "brightness-medium" => DARK_BRIGHTNESS_MEDIUM_SVG, LIGHT_BRIGHTNESS_MEDIUM_SVG;
    "broom" => DARK_BROOM_SVG, LIGHT_BROOM_SVG;
    "caffeine" => DARK_CAFFEINE_SVG, LIGHT_CAFFEINE_SVG;
    "calendar" => DARK_CALENDAR_SVG, LIGHT_CALENDAR_SVG;
    "camera" => DARK_CAMERA_SVG, LIGHT_CAMERA_SVG;
    "check" => DARK_CHECK_SVG, LIGHT_CHECK_SVG;
    "clock" => DARK_CLOCK_SVG, LIGHT_CLOCK_SVG;
    "close" => DARK_CLOSE_SVG, LIGHT_CLOSE_SVG;
    "copy" => DARK_COPY_SVG, LIGHT_COPY_SVG;
    "cut" => DARK_CUT_SVG, LIGHT_CUT_SVG;
    "dark-mode" => DARK_DARK_MODE_SVG, LIGHT_DARK_MODE_SVG;
    "display" => DARK_DISPLAY_SVG, LIGHT_DISPLAY_SVG;
    "down" => DARK_DOWN_SVG, LIGHT_DOWN_SVG;
    "download" => DARK_DOWNLOAD_SVG, LIGHT_DOWNLOAD_SVG;
    "drive-harddisk" => DARK_DRIVE_HARDDISK_SVG, LIGHT_DRIVE_HARDDISK_SVG;
    "edit" => DARK_EDIT_SVG, LIGHT_EDIT_SVG;
    "ellipse" => DARK_ELLIPSE_SVG, LIGHT_ELLIPSE_SVG;
    "eraser" => DARK_ERASER_SVG, LIGHT_ERASER_SVG;
    "ethernet" => DARK_ETHERNET_SVG, LIGHT_ETHERNET_SVG;
    "external-link" => DARK_EXTERNAL_LINK_SVG, LIGHT_EXTERNAL_LINK_SVG;
    "eye" => DARK_EYE_SVG, LIGHT_EYE_SVG;
    "eye-off" => DARK_EYE_OFF_SVG, LIGHT_EYE_OFF_SVG;
    "folder" => DARK_FOLDER_SVG, LIGHT_FOLDER_SVG;
    "folder-desktop" => DARK_FOLDER_DESKTOP_SVG, LIGHT_FOLDER_DESKTOP_SVG;
    "folder-documents" => DARK_FOLDER_DOCUMENTS_SVG, LIGHT_FOLDER_DOCUMENTS_SVG;
    "folder-download" => DARK_FOLDER_DOWNLOAD_SVG, LIGHT_FOLDER_DOWNLOAD_SVG;
    "folder-music" => DARK_FOLDER_MUSIC_SVG, LIGHT_FOLDER_MUSIC_SVG;
    "folder-new" => DARK_FOLDER_NEW_SVG, LIGHT_FOLDER_NEW_SVG;
    "folder-pictures" => DARK_FOLDER_PICTURES_SVG, LIGHT_FOLDER_PICTURES_SVG;
    "folder-videos" => DARK_FOLDER_VIDEOS_SVG, LIGHT_FOLDER_VIDEOS_SVG;
    "forward" => DARK_FORWARD_SVG, LIGHT_FORWARD_SVG;
    "gsconnect" => DARK_GSCONNECT_SVG, LIGHT_GSCONNECT_SVG;
    "info" => DARK_INFO_SVG, LIGHT_INFO_SVG;
    "line" => DARK_LINE_SVG, LIGHT_LINE_SVG;
    "lock" => DARK_LOCK_SVG, LIGHT_LOCK_SVG;
    "logo" => DARK_LOGO_SVG, LIGHT_LOGO_SVG;
    "logout" => DARK_LOGOUT_SVG, LIGHT_LOGOUT_SVG;
    "microphone" => DARK_MICROPHONE_SVG, LIGHT_MICROPHONE_SVG;
    "minus" => DARK_MINUS_SVG, LIGHT_MINUS_SVG;
    "music" => DARK_MUSIC_SVG, LIGHT_MUSIC_SVG;
    "next" => DARK_NEXT_SVG, LIGHT_NEXT_SVG;
    "night-light" => DARK_NIGHT_LIGHT_SVG, LIGHT_NIGHT_LIGHT_SVG;
    "paste" => DARK_PASTE_SVG, LIGHT_PASTE_SVG;
    "pause" => DARK_PAUSE_SVG, LIGHT_PAUSE_SVG;
    "performance" => DARK_PERFORMANCE_SVG, LIGHT_PERFORMANCE_SVG;
    "play" => DARK_PLAY_SVG, LIGHT_PLAY_SVG;
    "plus" => DARK_PLUS_SVG, LIGHT_PLUS_SVG;
    "power" => DARK_POWER_SVG, LIGHT_POWER_SVG;
    "previous" => DARK_PREVIOUS_SVG, LIGHT_PREVIOUS_SVG;
    "privacy" => DARK_PRIVACY_SVG, LIGHT_PRIVACY_SVG;
    "rect" => DARK_RECT_SVG, LIGHT_RECT_SVG;
    "refresh" => DARK_REFRESH_SVG, LIGHT_REFRESH_SVG;
    "rename" => DARK_RENAME_SVG, LIGHT_RENAME_SVG;
    "restart" => DARK_RESTART_SVG, LIGHT_RESTART_SVG;
    "search" => DARK_SEARCH_SVG, LIGHT_SEARCH_SVG;
    "server" => DARK_SERVER_SVG, LIGHT_SERVER_SVG;
    "settings" => DARK_SETTINGS_SVG, LIGHT_SETTINGS_SVG;
    "shield" => DARK_SHIELD_SVG, LIGHT_SHIELD_SVG;
    "sidebar" => DARK_SIDEBAR_SVG, LIGHT_SIDEBAR_SVG;
    "sleep" => DARK_SLEEP_SVG, LIGHT_SLEEP_SVG;
    "stop" => DARK_STOP_SVG, LIGHT_STOP_SVG;
    "terminal" => DARK_TERMINAL_SVG, LIGHT_TERMINAL_SVG;
    "text" => DARK_TEXT_SVG, LIGHT_TEXT_SVG;
    "trash" => DARK_TRASH_SVG, LIGHT_TRASH_SVG;
    "unlock" => DARK_UNLOCK_SVG, LIGHT_UNLOCK_SVG;
    "up" => DARK_UP_SVG, LIGHT_UP_SVG;
    "user" => DARK_USER_SVG, LIGHT_USER_SVG;
    "user-home" => DARK_USER_HOME_SVG, LIGHT_USER_HOME_SVG;
    "user-trash" => DARK_USER_TRASH_SVG, LIGHT_USER_TRASH_SVG;
    "view-grid" => DARK_VIEW_GRID_SVG, LIGHT_VIEW_GRID_SVG;
    "view-list" => DARK_VIEW_LIST_SVG, LIGHT_VIEW_LIST_SVG;
    "volume" => DARK_VOLUME_SVG, LIGHT_VOLUME_SVG;
    "volume-low" => DARK_VOLUME_LOW_SVG, LIGHT_VOLUME_LOW_SVG;
    "volume-mute" => DARK_VOLUME_MUTE_SVG, LIGHT_VOLUME_MUTE_SVG;
    "wifi" => DARK_WIFI_SVG, LIGHT_WIFI_SVG;
    "zoom-fit" => DARK_ZOOM_FIT_SVG, LIGHT_ZOOM_FIT_SVG;
    "zoom-in" => DARK_ZOOM_IN_SVG, LIGHT_ZOOM_IN_SVG;
    "zoom-out" => DARK_ZOOM_OUT_SVG, LIGHT_ZOOM_OUT_SVG;
}
