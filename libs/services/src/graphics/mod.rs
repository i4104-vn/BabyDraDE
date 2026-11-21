//! Shared Cairo drawing primitives used by compositor overlays and UI features.
//!
//! Keeping these here avoids duplicating rounded paths, text helpers and
//! per-app accent colours in every shell extension.

mod draw;

pub use draw::{
    app_color, docked_panel_path, draw_docked_panel, draw_text, fit_text, paint_rounded,
    rounded_rect, set_bold_font, DockSide,
};
