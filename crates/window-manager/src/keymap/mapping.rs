//! Combo parsing and XKB keysym mapping for configurable shortcuts.

use serde::{Deserialize, Serialize};
use smithay::input::keyboard::keysyms;

fn default_true() -> bool {
    true
}

/// A global keyboard shortcut loaded from the config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortcut {
    pub id: usize,
    pub modifiers: String,
    pub key: String,
    pub command: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Modifier set required for a shortcut.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Mods {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub win: bool,
}

/// A shortcut resolved into a keysym + required modifier set.
#[derive(Clone, Debug)]
pub struct ResolvedShortcut {
    pub mods: Mods,
    pub keysym: u32,
    pub command: String,
}

impl ResolvedShortcut {
    /// Converts a shared `Shortcut`; `None` when unparseable.
    pub fn resolve(shortcut: &Shortcut) -> Option<Self> {
        if !shortcut.enabled {
            return None;
        }
        let mods = parse_modifiers(&shortcut.modifiers)?;
        let keysym = key_name_to_keysym(&shortcut.key)?;
        Some(Self {
            mods,
            keysym,
            command: shortcut.command.clone(),
        })
    }
}

/// Parses a modifier string like `"S"` or `"W-S"`.
fn parse_modifiers(spec: &str) -> Option<Mods> {
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

/// Maps a config key name (e.g. `"Tab"`, `"F12"`, `"s"`) to the XKB keysym
/// produced by `KeysymHandle::raw_latin_sym_or_raw_current_sym`.
pub fn key_name_to_keysym(name: &str) -> Option<u32> {
    let up = name.to_ascii_uppercase();

    // Single characters -> lowercase letters / digits.
    if up.len() == 1 {
        let c = up.chars().next()?;
        if c.is_ascii_alphabetic() {
            return Some(keysyms::KEY_a + (c as u32 - 'A' as u32));
        }
        if c.is_ascii_digit() {
            return Some(c as u32);
        }
    }

    // F-keys: F1 .. F12
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

    #[test]
    fn maps_letters_to_lowercase_keysyms() {
        assert_eq!(key_name_to_keysym("s"), Some(keysyms::KEY_s));
        assert_eq!(key_name_to_keysym("A"), Some(keysyms::KEY_a));
        assert_eq!(key_name_to_keysym("f12"), Some(keysyms::KEY_F12));
        assert_eq!(key_name_to_keysym("Tab"), Some(keysyms::KEY_Tab));
        assert_eq!(key_name_to_keysym("nope"), None);
    }

    #[test]
    fn resolves_modifiers_and_key() {
        let shortcut = Shortcut {
            id: 1,
            modifiers: "W-S".into(),
            key: "s".into(),
            command: "echo hi".into(),
            enabled: true,
        };
        let resolved = ResolvedShortcut::resolve(&shortcut).unwrap();
        assert_eq!(resolved.keysym, keysyms::KEY_s);
        assert_eq!(
            resolved.mods,
            Mods {
                shift: true,
                win: true,
                ..Mods::default()
            }
        );
    }
}
