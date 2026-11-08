//! Fluent Design tokens and color themes for BabyDraDE UI Kit

#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub bg_glass: &'static str,
    pub border_subtle: &'static str,
    pub border_hover: &'static str,
    pub text_primary: &'static str,
    pub text_secondary: &'static str,
    pub text_muted: &'static str,
    pub accent_arch: &'static str,
    pub accent_arch_subtle: &'static str,
    pub shadow_pill: &'static str,
}

pub const FLUENT_DARK: ThemePalette = ThemePalette {
    bg_glass: "rgba(24, 26, 32, 0.85)",
    border_subtle: "rgba(255, 255, 255, 0.12)",
    border_hover: "rgba(255, 255, 255, 0.22)",
    text_primary: "#FFFFFF",
    text_secondary: "#F3F4F6",
    text_muted: "rgba(255, 255, 255, 0.25)",
    accent_arch: "#1793D1",
    accent_arch_subtle: "rgba(56, 185, 240, 0.20)",
    shadow_pill: "0 4px 14px rgba(0, 0, 0, 0.4)",
};

pub const FLUENT_LIGHT: ThemePalette = ThemePalette {
    bg_glass: "rgba(255, 255, 255, 0.85)",
    border_subtle: "rgba(0, 0, 0, 0.08)",
    border_hover: "rgba(0, 0, 0, 0.16)",
    text_primary: "#111827",
    text_secondary: "#1F2937",
    text_muted: "rgba(0, 0, 0, 0.25)",
    accent_arch: "#0078D4",
    accent_arch_subtle: "rgba(0, 120, 212, 0.15)",
    shadow_pill: "0 4px 14px rgba(0, 0, 0, 0.08)",
};

