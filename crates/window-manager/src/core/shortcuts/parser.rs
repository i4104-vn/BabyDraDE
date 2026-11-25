//! Shortcut parsing rules.

use super::domain::Mods;
use crate::shared::types::keysyms;

/// Parses a modifier specification such as `"S"` or `"W-S"`.
///
/// # Arguments
///
/// * `spec` - Dash-separated modifier tokens (`S`, `C`, `A`, `W`).
///
/// # Returns
///
/// `None` if any token is unknown.
pub(crate) fn parse_modifiers(spec: &str) -> Option<Mods> {
    let mut mods = Mods::default();
    for token in spec.split('-') {
        match token.to_ascii_uppercase().as_str() {
            "" => continue,
            "S" => mods.shift = true,
            "C" => mods.ctrl = true,
            "A" => mods.alt = true,
            "W" => mods.win = true,
            _ => return None,
        }
    }
    Some(mods)
}

/// Splits a keymap combo such as `"W-S-t"` into `("W-S", "t")`.
///
/// A global shortcut needs at least one modifier, so combos without a dash
/// or with an empty side are rejected.
///
/// # Arguments
///
/// * `combo` - Raw combo string from config or a registration request.
///
/// # Returns
///
/// The modifier spec and key name, or `None` when the combo is malformed.
pub(crate) fn parse_combo(combo: &str) -> Option<(String, String)> {
    let (modifiers, key) = combo.rsplit_once('-')?;
    if modifiers.is_empty() || key.is_empty() {
        return None;
    }
    Some((modifiers.to_string(), key.to_string()))
}

/// Maps a config key name to an XKB keysym.
///
/// # Arguments
///
/// * `name` - Key name such as `"Tab"`, `"F12"` or `"s"`.
///
/// # Returns
///
/// The keysym, or `None` when the name is not recognised.
pub(crate) fn to_keysym(name: &str) -> Option<u32> {
    let up = name.to_ascii_uppercase();

    if up.len() == 1 {
        let c = up.chars().next()?;
        if c.is_ascii_alphabetic() {
            return Some(keysyms::KEY_a + (c as u32 - 'A' as u32));
        }
        if c.is_ascii_digit() {
            return Some(c as u32);
        }
    }

    if let Some(num) = up.strip_prefix('F') {
        if let Ok(n) = num.parse::<u8>() {
            if (1..=12).contains(&n) {
                return Some(f_key(n));
            }
        }
    }

    Some(match up.as_str() {
        "TAB" => keysyms::KEY_Tab,
        "SPACE" => keysyms::KEY_space,
        "ENTER" | "RETURN" => keysyms::KEY_Return,
        "ESC" | "ESCAPE" => keysyms::KEY_Escape,
        "BACKSPACE" | "BKSP" => keysyms::KEY_BackSpace,
        "DELETE" | "DEL" => keysyms::KEY_Delete,
        "INSERT" => keysyms::KEY_Insert,
        "HOME" => keysyms::KEY_Home,
        "END" => keysyms::KEY_End,
        "PAGEUP" | "PRIOR" => keysyms::KEY_Prior,
        "PAGEDOWN" | "NEXT" => keysyms::KEY_Next,
        "UP" => keysyms::KEY_Up,
        "DOWN" => keysyms::KEY_Down,
        "LEFT" => keysyms::KEY_Left,
        "RIGHT" => keysyms::KEY_Right,
        "MINUS" => keysyms::KEY_minus,
        "EQUAL" => keysyms::KEY_equal,
        "COMMA" => keysyms::KEY_comma,
        "DOT" | "PERIOD" => keysyms::KEY_period,
        "SLASH" => keysyms::KEY_slash,
        "SEMICOLON" => keysyms::KEY_semicolon,
        "APOSTROPHE" => keysyms::KEY_apostrophe,
        "GRAVE" | "BACKQUOTE" => keysyms::KEY_grave,
        "BACKSLASH" => keysyms::KEY_backslash,
        "LEFTBRACKET" | "BRACKETLEFT" => keysyms::KEY_bracketleft,
        "RIGHTBRACKET" | "BRACKETRIGHT" => keysyms::KEY_bracketright,
        "PRINT" | "PRINTSCREEN" => keysyms::KEY_Print,
        "PAUSE" => keysyms::KEY_Pause,
        "MUTE" => keysyms::KEY_XF86AudioMute,
        "VOLUMEUP" => keysyms::KEY_XF86AudioRaiseVolume,
        "VOLUMEDOWN" => keysyms::KEY_XF86AudioLowerVolume,
        "PLAYPAUSE" => keysyms::KEY_XF86AudioPlay,
        "PREVIOUSSONG" => keysyms::KEY_XF86AudioPrev,
        "NEXTSONG" => keysyms::KEY_XF86AudioNext,
        _ => return None,
    })
}

fn f_key(n: u8) -> u32 {
    keysyms::KEY_F1 + (n as u32 - 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::types::keysyms;

    #[test]
    fn splits_combos_into_modifiers_and_key() {
        assert_eq!(
            parse_combo("W-S-t"),
            Some(("W-S".to_string(), "t".to_string()))
        );
        assert_eq!(parse_combo("A-F4"), Some(("A".to_string(), "F4".to_string())));
        assert_eq!(parse_combo("t"), None);
        assert_eq!(parse_combo("-t"), None);
        assert_eq!(parse_combo("W-"), None);
    }

    #[test]
    fn maps_letters_to_lowercase_keysyms() {
        assert_eq!(to_keysym("s"), Some(keysyms::KEY_s));
        assert_eq!(to_keysym("A"), Some(keysyms::KEY_a));
        assert_eq!(to_keysym("f12"), Some(keysyms::KEY_F12));
        assert_eq!(to_keysym("Tab"), Some(keysyms::KEY_Tab));
        assert_eq!(to_keysym("nope"), None);
    }
}
