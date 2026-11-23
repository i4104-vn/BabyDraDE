//! Shortcut entities and rules (pure domain logic).
//!
//! This module knows nothing about config files, processes or the event loop.
//! It owns only the data and the matching rule; keysym identifiers come from
//! the framework-free [`super::keysyms`] table.

use super::keysyms;
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

/// One configured global shortcut: a key combo and the command it runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Shortcut {
    pub(crate) id: usize,
    pub(crate) modifiers: String,
    pub(crate) key: String,
    pub(crate) command: String,
    #[serde(default = "default_true")]
    pub(crate) enabled: bool,
}

/// Modifier mask required for a shortcut to match a key press.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct Mods {
    pub(crate) shift: bool,
    pub(crate) ctrl: bool,
    pub(crate) alt: bool,
    pub(crate) win: bool,
}

/// A shortcut parsed into a keysym and its required modifier mask.
#[derive(Clone, Debug)]
pub(crate) struct ResolvedShortcut {
    pub(crate) mods: Mods,
    pub(crate) keysym: u32,
    pub(crate) command: String,
}

impl ResolvedShortcut {
    /// Parses a configured shortcut into a comparable keysym and modifier mask.
    ///
    /// # Arguments
    ///
    /// * `shortcut` - Raw configuration entry.
    ///
    /// # Returns
    ///
    /// `None` when the entry is disabled or its combo cannot be parsed.
    pub(crate) fn resolve(shortcut: &Shortcut) -> Option<Self> {
        if !shortcut.enabled {
            return None;
        }
        Some(Self {
            mods: parse_modifiers(&shortcut.modifiers)?,
            keysym: key_name_to_keysym(&shortcut.key)?,
            command: shortcut.command.clone(),
        })
    }
}

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

/// Finds the command bound to an exact key press.
///
/// # Arguments
///
/// * `mods` - Modifier mask active during the key press.
/// * `keysym` - Layout-independent XKB keysym.
/// * `shortcuts` - Configured shortcuts, searched in order.
///
/// # Returns
///
/// The command of the first exact match, or `None` when nothing matches.
pub(crate) fn command_for(mods: Mods, keysym: u32, shortcuts: &[Shortcut]) -> Option<String> {
    shortcuts
        .iter()
        .filter_map(ResolvedShortcut::resolve)
        .find(|s| s.keysym == keysym && s.mods == mods)
        .map(|s| s.command)
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
pub(crate) fn key_name_to_keysym(name: &str) -> Option<u32> {
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
#[path = "../tests/domain/shortcuts.rs"]
mod tests;
