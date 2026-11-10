//! BabyDraDE UI Kit — Reusable GTK4 Components, Icons, and Fluent Stylesheets

pub mod components;
pub mod drawing;
pub mod icons;
pub mod locale;
pub mod style;
pub mod theme;

pub use components::*;
pub use drawing::*;
pub use icons::{get_fallback_icon, get_icon, get_icon_colored, get_logo_png, get_theme_or_svg_icon, set_fallback_icon, set_image_from_icon};
pub use services::i18n;
pub use services::i18n::trans;
pub use locale::watch_locale_change;
pub use style::load_ui_kit_styles;
pub use theme::{init_theme, is_dark_mode, set_dark_mode};
