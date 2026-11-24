//! Icon asset pairs for names starting with g_m.

pub(super) fn resolve(name: &str) -> Option<(&'static str, &'static str)> {
    match name {
        "gsconnect" => Some((
            crate::icons::assets::DARK_GSCONNECT_SVG,
            crate::icons::assets::LIGHT_GSCONNECT_SVG,
        )),
        "info" => Some((crate::icons::assets::DARK_INFO_SVG, crate::icons::assets::LIGHT_INFO_SVG)),
        "line" => Some((crate::icons::assets::DARK_LINE_SVG, crate::icons::assets::LIGHT_LINE_SVG)),
        "lock" => Some((crate::icons::assets::DARK_LOCK_SVG, crate::icons::assets::LIGHT_LOCK_SVG)),
        "logo" => Some((crate::icons::assets::DARK_LOGO_SVG, crate::icons::assets::LIGHT_LOGO_SVG)),
        "logout" => Some((
            crate::icons::assets::DARK_LOGOUT_SVG,
            crate::icons::assets::LIGHT_LOGOUT_SVG,
        )),
        "microphone" => Some((
            crate::icons::assets::DARK_MICROPHONE_SVG,
            crate::icons::assets::LIGHT_MICROPHONE_SVG,
        )),
        "minus" => Some((
            crate::icons::assets::DARK_MINUS_SVG,
            crate::icons::assets::LIGHT_MINUS_SVG,
        )),
        "music" => Some((
            crate::icons::assets::DARK_MUSIC_SVG,
            crate::icons::assets::LIGHT_MUSIC_SVG,
        )),
        "history" => Some((
            crate::icons::assets::DARK_REFRESH_SVG,
            crate::icons::assets::LIGHT_REFRESH_SVG,
        )),
        "keybinds" => Some((
            crate::icons::assets::DARK_SETTINGS_SVG,
            crate::icons::assets::LIGHT_SETTINGS_SVG,
        )),
        _ => None,
    }
}
