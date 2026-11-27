//! BabyDraDE UI Kit — Reusable GTK4 Components, Icons, and Fluent Stylesheets

pub mod components;
pub mod drawing;
pub mod icons;
pub mod layer_shell;
pub mod locale;
pub mod style;
pub mod theme;

pub use components::*;
pub use drawing::*;
pub use icons::{
    get_app_icon_pixbuf, get_app_icon_pixbuf_any, get_app_icon_pixbuf_any_for_window,
    get_app_icon_pixbuf_for_window, get_fallback_icon, get_icon, get_icon_colored,
    get_icon_pixbuf_from_filesystem, get_logo_png, get_theme_icon_pixbuf, get_theme_or_svg_icon,
    pixbuf_from_gicon, resolve_app_gicon, resolve_app_gicon_for_window, resolve_app_icon_name,
    resolve_app_icon_name_for_window, set_fallback_icon, set_image_from_icon,
};
pub use locale::watch_locale_change;
pub use services::i18n;
pub use services::i18n::trans;
pub use style::load_ui_kit_styles;
pub use theme::{init_theme, is_dark_mode, set_dark_mode};
