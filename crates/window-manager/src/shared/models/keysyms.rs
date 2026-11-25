//! XKB keysym identifiers used by configurable shortcuts.
//!
//! Names mirror the XKB constants (`KEY_a`, `KEY_Tab`, ...) so config mapping
//! reads like the protocol definition. Owning these integer constants keeps
//! shortcut matching independent of a window-system crate.
#![allow(non_upper_case_globals)]

pub(crate) const KEY_space: u32 = 0x0020;
pub(crate) const KEY_apostrophe: u32 = 0x0027;
pub(crate) const KEY_comma: u32 = 0x002c;
pub(crate) const KEY_minus: u32 = 0x002d;
pub(crate) const KEY_period: u32 = 0x002e;
pub(crate) const KEY_slash: u32 = 0x002f;
pub(crate) const KEY_semicolon: u32 = 0x003b;
pub(crate) const KEY_equal: u32 = 0x003d;
pub(crate) const KEY_bracketleft: u32 = 0x005b;
pub(crate) const KEY_backslash: u32 = 0x005c;
pub(crate) const KEY_bracketright: u32 = 0x005d;
pub(crate) const KEY_grave: u32 = 0x0060;
pub(crate) const KEY_a: u32 = 0x0061;
#[cfg(test)]
pub(crate) const KEY_e: u32 = 0x0065;
#[cfg(test)]
pub(crate) const KEY_s: u32 = 0x0073;
#[cfg(test)]
pub(crate) const KEY_t: u32 = 0x0074;

pub(crate) const KEY_BackSpace: u32 = 0xff08;
pub(crate) const KEY_Tab: u32 = 0xff09;
pub(crate) const KEY_Return: u32 = 0xff0d;
pub(crate) const KEY_Pause: u32 = 0xff13;
pub(crate) const KEY_Escape: u32 = 0xff1b;
pub(crate) const KEY_Delete: u32 = 0xffff;
pub(crate) const KEY_Home: u32 = 0xff50;
pub(crate) const KEY_Left: u32 = 0xff51;
pub(crate) const KEY_Up: u32 = 0xff52;
pub(crate) const KEY_Right: u32 = 0xff53;
pub(crate) const KEY_Down: u32 = 0xff54;
pub(crate) const KEY_Prior: u32 = 0xff55;
pub(crate) const KEY_Next: u32 = 0xff56;
pub(crate) const KEY_End: u32 = 0xff57;
pub(crate) const KEY_Print: u32 = 0xff61;
pub(crate) const KEY_Insert: u32 = 0xff63;
pub(crate) const KEY_F1: u32 = 0xffbe;
#[cfg(test)]
pub(crate) const KEY_F12: u32 = 0xffc9;

pub(crate) const KEY_XF86AudioLowerVolume: u32 = 0x1008_ff11;
pub(crate) const KEY_XF86AudioMute: u32 = 0x1008_ff12;
pub(crate) const KEY_XF86AudioRaiseVolume: u32 = 0x1008_ff13;
pub(crate) const KEY_XF86AudioPlay: u32 = 0x1008_ff14;
pub(crate) const KEY_XF86AudioPrev: u32 = 0x1008_ff16;
pub(crate) const KEY_XF86AudioNext: u32 = 0x1008_ff17;
