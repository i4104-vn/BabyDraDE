//! Icon asset pairs for names starting with n_s.

pub(super) fn resolve(name: &str) -> Option<(&'static str, &'static str)> {
    match name {
        "next" => Some((
            crate::icons::assets::DARK_NEXT_SVG,
            crate::icons::assets::LIGHT_NEXT_SVG,
        )),
        "night-light" => Some((
            crate::icons::assets::DARK_NIGHT_LIGHT_SVG,
            crate::icons::assets::LIGHT_NIGHT_LIGHT_SVG,
        )),
        "paste" => Some((
            crate::icons::assets::DARK_PASTE_SVG,
            crate::icons::assets::LIGHT_PASTE_SVG,
        )),
        "pause" => Some((
            crate::icons::assets::DARK_PAUSE_SVG,
            crate::icons::assets::LIGHT_PAUSE_SVG,
        )),
        "performance" => Some((
            crate::icons::assets::DARK_PERFORMANCE_SVG,
            crate::icons::assets::LIGHT_PERFORMANCE_SVG,
        )),
        "play" => Some((
            crate::icons::assets::DARK_PLAY_SVG,
            crate::icons::assets::LIGHT_PLAY_SVG,
        )),
        "plus" => Some((
            crate::icons::assets::DARK_PLUS_SVG,
            crate::icons::assets::LIGHT_PLUS_SVG,
        )),
        "power" => Some((
            crate::icons::assets::DARK_POWER_SVG,
            crate::icons::assets::LIGHT_POWER_SVG,
        )),
        "previous" => Some((
            crate::icons::assets::DARK_PREVIOUS_SVG,
            crate::icons::assets::LIGHT_PREVIOUS_SVG,
        )),
        "privacy" => Some((
            crate::icons::assets::DARK_PRIVACY_SVG,
            crate::icons::assets::LIGHT_PRIVACY_SVG,
        )),
        "rect" => Some((
            crate::icons::assets::DARK_RECT_SVG,
            crate::icons::assets::LIGHT_RECT_SVG,
        )),
        "refresh" => Some((
            crate::icons::assets::DARK_REFRESH_SVG,
            crate::icons::assets::LIGHT_REFRESH_SVG,
        )),
        "rename" => Some((
            crate::icons::assets::DARK_RENAME_SVG,
            crate::icons::assets::LIGHT_RENAME_SVG,
        )),
        "restart" => Some((
            crate::icons::assets::DARK_RESTART_SVG,
            crate::icons::assets::LIGHT_RESTART_SVG,
        )),
        "search" => Some((
            crate::icons::assets::DARK_SEARCH_SVG,
            crate::icons::assets::LIGHT_SEARCH_SVG,
        )),
        "server" => Some((
            crate::icons::assets::DARK_SERVER_SVG,
            crate::icons::assets::LIGHT_SERVER_SVG,
        )),
        "settings" => Some((
            crate::icons::assets::DARK_SETTINGS_SVG,
            crate::icons::assets::LIGHT_SETTINGS_SVG,
        )),
        "shield" => Some((
            crate::icons::assets::DARK_SHIELD_SVG,
            crate::icons::assets::LIGHT_SHIELD_SVG,
        )),
        "sidebar" => Some((
            crate::icons::assets::DARK_SIDEBAR_SVG,
            crate::icons::assets::LIGHT_SIDEBAR_SVG,
        )),
        "sleep" => Some((
            crate::icons::assets::DARK_SLEEP_SVG,
            crate::icons::assets::LIGHT_SLEEP_SVG,
        )),
        "stop" => Some((
            crate::icons::assets::DARK_STOP_SVG,
            crate::icons::assets::LIGHT_STOP_SVG,
        )),
        "palette" => Some((
            crate::icons::assets::DARK_DISPLAY_SVG,
            crate::icons::assets::LIGHT_DISPLAY_SVG,
        )),
        "sliders" => Some((
            crate::icons::assets::DARK_SETTINGS_SVG,
            crate::icons::assets::LIGHT_SETTINGS_SVG,
        )),
        "square" => Some((
            crate::icons::assets::DARK_STOP_SVG,
            crate::icons::assets::LIGHT_STOP_SVG,
        )),
        _ => None,
    }
}
