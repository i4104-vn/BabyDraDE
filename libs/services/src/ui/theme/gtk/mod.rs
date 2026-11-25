/// GTK4 CSS file discovery.
pub mod css;
/// GTK settings.ini parsing.
pub mod reader;

pub use css::{find_gtk4_css_path, read_gtk4_css};
pub use reader::{find_settings_ini_path, parse_settings_ini, read_gtk_settings};
