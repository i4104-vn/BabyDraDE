//! Icon asset pairs for names starting with a_c.

pub(super) fn resolve(name: &str) -> Option<(&'static str, &'static str)> {
    match name {
        "activity" => Some((
            crate::icons::assets::DARK_ACTIVITY_SVG,
            crate::icons::assets::LIGHT_ACTIVITY_SVG,
        )),
        "airplane" => Some((
            crate::icons::assets::DARK_AIRPLANE_SVG,
            crate::icons::assets::LIGHT_AIRPLANE_SVG,
        )),
        "arrow" => Some((
            crate::icons::assets::DARK_ARROW_SVG,
            crate::icons::assets::LIGHT_ARROW_SVG,
        )),
        "back" => Some((crate::icons::assets::DARK_BACK_SVG, crate::icons::assets::LIGHT_BACK_SVG)),
        "battery" => Some((
            crate::icons::assets::DARK_BATTERY_SVG,
            crate::icons::assets::LIGHT_BATTERY_SVG,
        )),
        "bell" => Some((crate::icons::assets::DARK_BELL_SVG, crate::icons::assets::LIGHT_BELL_SVG)),
        "bell-off" => Some((
            crate::icons::assets::DARK_BELL_OFF_SVG,
            crate::icons::assets::LIGHT_BELL_OFF_SVG,
        )),
        "bluetooth" => Some((
            crate::icons::assets::DARK_BLUETOOTH_SVG,
            crate::icons::assets::LIGHT_BLUETOOTH_SVG,
        )),
        "blur" => Some((crate::icons::assets::DARK_BLUR_SVG, crate::icons::assets::LIGHT_BLUR_SVG)),
        "brightness" => Some((
            crate::icons::assets::DARK_BRIGHTNESS_SVG,
            crate::icons::assets::LIGHT_BRIGHTNESS_SVG,
        )),
        "brightness-low" => Some((
            crate::icons::assets::DARK_BRIGHTNESS_LOW_SVG,
            crate::icons::assets::LIGHT_BRIGHTNESS_LOW_SVG,
        )),
        "brightness-medium" => Some((
            crate::icons::assets::DARK_BRIGHTNESS_MEDIUM_SVG,
            crate::icons::assets::LIGHT_BRIGHTNESS_MEDIUM_SVG,
        )),
        "broom" => Some((
            crate::icons::assets::DARK_BROOM_SVG,
            crate::icons::assets::LIGHT_BROOM_SVG,
        )),
        "caffeine" => Some((
            crate::icons::assets::DARK_CAFFEINE_SVG,
            crate::icons::assets::LIGHT_CAFFEINE_SVG,
        )),
        "calendar" => Some((
            crate::icons::assets::DARK_CALENDAR_SVG,
            crate::icons::assets::LIGHT_CALENDAR_SVG,
        )),
        "camera" => Some((
            crate::icons::assets::DARK_CAMERA_SVG,
            crate::icons::assets::LIGHT_CAMERA_SVG,
        )),
        "check" => Some((
            crate::icons::assets::DARK_CHECK_SVG,
            crate::icons::assets::LIGHT_CHECK_SVG,
        )),
        "clock" => Some((
            crate::icons::assets::DARK_CLOCK_SVG,
            crate::icons::assets::LIGHT_CLOCK_SVG,
        )),
        "close" => Some((
            crate::icons::assets::DARK_CLOSE_SVG,
            crate::icons::assets::LIGHT_CLOSE_SVG,
        )),
        "copy" => Some((crate::icons::assets::DARK_COPY_SVG, crate::icons::assets::LIGHT_COPY_SVG)),
        "cut" => Some((crate::icons::assets::DARK_CUT_SVG, crate::icons::assets::LIGHT_CUT_SVG)),
        "cog" => Some((
            crate::icons::assets::DARK_SETTINGS_SVG,
            crate::icons::assets::LIGHT_SETTINGS_SVG,
        )),
        "avatar-default" => Some((crate::icons::assets::DARK_USER_SVG, crate::icons::assets::LIGHT_USER_SVG)),
        "circle" => Some((
            crate::icons::assets::DARK_ELLIPSE_SVG,
            crate::icons::assets::LIGHT_ELLIPSE_SVG,
        )),
        _ => None,
    }
}
