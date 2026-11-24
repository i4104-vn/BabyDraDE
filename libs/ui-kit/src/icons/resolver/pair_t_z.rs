//! Icon asset pairs for names starting with t_z.

pub(super) fn resolve(name: &str) -> Option<(&'static str, &'static str)> {
    match name {
        "terminal" => Some((
            crate::icons::assets::DARK_TERMINAL_SVG,
            crate::icons::assets::LIGHT_TERMINAL_SVG,
        )),
        "text" => Some((crate::icons::assets::DARK_TEXT_SVG, crate::icons::assets::LIGHT_TEXT_SVG)),
        "trash" => Some((
            crate::icons::assets::DARK_TRASH_SVG,
            crate::icons::assets::LIGHT_TRASH_SVG,
        )),
        "unlock" => Some((
            crate::icons::assets::DARK_UNLOCK_SVG,
            crate::icons::assets::LIGHT_UNLOCK_SVG,
        )),
        "up" => Some((crate::icons::assets::DARK_UP_SVG, crate::icons::assets::LIGHT_UP_SVG)),
        "user" => Some((crate::icons::assets::DARK_USER_SVG, crate::icons::assets::LIGHT_USER_SVG)),
        "user-home" => Some((
            crate::icons::assets::DARK_USER_HOME_SVG,
            crate::icons::assets::LIGHT_USER_HOME_SVG,
        )),
        "user-trash" => Some((
            crate::icons::assets::DARK_USER_TRASH_SVG,
            crate::icons::assets::LIGHT_USER_TRASH_SVG,
        )),
        "view-grid" => Some((
            crate::icons::assets::DARK_VIEW_GRID_SVG,
            crate::icons::assets::LIGHT_VIEW_GRID_SVG,
        )),
        "view-list" => Some((
            crate::icons::assets::DARK_VIEW_LIST_SVG,
            crate::icons::assets::LIGHT_VIEW_LIST_SVG,
        )),
        "volume" => Some((
            crate::icons::assets::DARK_VOLUME_SVG,
            crate::icons::assets::LIGHT_VOLUME_SVG,
        )),
        "volume-low" => Some((
            crate::icons::assets::DARK_VOLUME_LOW_SVG,
            crate::icons::assets::LIGHT_VOLUME_LOW_SVG,
        )),
        "volume-mute" => Some((
            crate::icons::assets::DARK_VOLUME_MUTE_SVG,
            crate::icons::assets::LIGHT_VOLUME_MUTE_SVG,
        )),
        "wifi" => Some((crate::icons::assets::DARK_WIFI_SVG, crate::icons::assets::LIGHT_WIFI_SVG)),
        "zoom-fit" => Some((
            crate::icons::assets::DARK_ZOOM_FIT_SVG,
            crate::icons::assets::LIGHT_ZOOM_FIT_SVG,
        )),
        "zoom-in" => Some((
            crate::icons::assets::DARK_ZOOM_IN_SVG,
            crate::icons::assets::LIGHT_ZOOM_IN_SVG,
        )),
        "zoom-out" => Some((
            crate::icons::assets::DARK_ZOOM_OUT_SVG,
            crate::icons::assets::LIGHT_ZOOM_OUT_SVG,
        )),
        "th-large" => Some((
            crate::icons::assets::DARK_VIEW_GRID_SVG,
            crate::icons::assets::LIGHT_VIEW_GRID_SVG,
        )),
        "zoom-original" => Some((
            crate::icons::assets::DARK_ZOOM_FIT_SVG,
            crate::icons::assets::LIGHT_ZOOM_FIT_SVG,
        )),
        "zoom-reset" => Some((
            crate::icons::assets::DARK_ZOOM_FIT_SVG,
            crate::icons::assets::LIGHT_ZOOM_FIT_SVG,
        )),
        _ => None,
    }
}
