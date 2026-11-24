//! System icon resolving logic and asset generation.

pub mod assets;
pub mod filesystem;
pub mod resolver;

use gtk4::gdk::Texture;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::gio::prelude::*;
use gtk4::gio::MemoryInputStream;
use gtk4::glib;

pub use crate::theme::is_dark_mode;
pub use filesystem::{get_icon_pixbuf_from_filesystem, pixbuf_from_gicon};
pub use resolver::resolve_icon_pair;

mod app;
mod image;
mod theme;
mod widget;

pub use app::{
    get_app_icon_pixbuf, get_app_icon_pixbuf_any, resolve_app_gicon, resolve_app_icon_name,
};
pub use image::{
    get_icon_colored, get_icon_from_svg, get_logo_paintable, get_logo_png, LOGO_BYTES,
};
pub use theme::get_theme_icon_pixbuf;
pub use widget::{
    get_fallback_icon, get_icon, get_theme_or_svg_icon, set_fallback_icon, set_image_from_icon,
};
