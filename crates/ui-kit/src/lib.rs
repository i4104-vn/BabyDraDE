//! BabyDraDE UI Kit — Reusable GTK4 Components, Icons, and Fluent Stylesheets

pub mod components;
pub mod drawing;
pub mod icons;
pub mod style;

pub use components::*;
pub use drawing::*;
pub use icons::{get_fallback_icon, get_icon, get_icon_colored, get_logo_png, get_theme_or_svg_icon, is_dark_mode, set_fallback_icon, set_image_from_icon};
pub use style::{load_ui_kit_styles, FLUENT_DARK, FLUENT_LIGHT};
