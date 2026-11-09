//! System icon resolving logic and asset generation.

pub mod assets;
pub mod resolver;

use gtk4::gdk::Texture;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::gio::prelude::*;
use gtk4::gio::MemoryInputStream;
use gtk4::glib;

pub use resolver::resolve_icon_pair;

/// Whether dark mode is currently active.
pub fn is_dark_mode() -> bool {
    if let Some(settings) = gtk4::Settings::default() {
        settings.is_gtk_application_prefer_dark_theme()
    } else {
        true
    }
}

/// Converts SVG string content into a `gtk4::Image` at the requested pixel size.
pub fn get_icon_from_svg(svg_content: &str, size: i32) -> gtk4::Image {
    let bytes = glib::Bytes::from(svg_content.as_bytes());
    let stream = MemoryInputStream::from_bytes(&bytes);

    let pixbuf = Pixbuf::from_stream_at_scale(&stream, size, size, true, gtk4::gio::Cancellable::NONE);

    match pixbuf {
        Ok(pb) => {
            let texture = Texture::for_pixbuf(&pb);
            let img = gtk4::Image::from_paintable(Some(&texture));
            img.set_pixel_size(size);
            img
        }
        Err(_) => gtk4::Image::from_icon_name("image-missing"),
    }
}

/// Embedded bytes of the BabyDra logo.
pub const LOGO_BYTES: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.png"));

/// Returns the embedded logo paintable at the requested size.
pub fn get_logo_paintable(size: i32) -> Option<Texture> {
    let bytes = glib::Bytes::from(LOGO_BYTES);
    let stream = MemoryInputStream::from_bytes(&bytes);
    Pixbuf::from_stream_at_scale(&stream, size, size, true, gtk4::gio::Cancellable::NONE)
        .ok()
        .map(|pb| Texture::for_pixbuf(&pb))
}

/// Returns the current `logo png` as a GTK Image.
pub fn get_logo_png(size: i32) -> gtk4::Image {
    if let Some(texture) = get_logo_paintable(size) {
        let img = gtk4::Image::from_paintable(Some(&texture));
        img.set_pixel_size(size);
        img
    } else {
        gtk4::Image::from_icon_name("image-missing")
    }
}

/// Returns the current `icon colored`.
pub fn get_icon_colored(name: &str, size: i32, color_hex: &str) -> gtk4::Image {
    let final_color = if !is_dark_mode() {
        let normalized = color_hex.replace(' ', "");
        match normalized.as_str() {
            "rgba(255,255,255,0.8)" | "rgba(255,255,255,0.80)" => "rgba(28, 28, 30, 0.85)".to_string(),
            "rgba(255,255,255,0.7)" | "rgba(255,255,255,0.70)" => "rgba(28, 28, 30, 0.75)".to_string(),
            "rgba(255,255,255,0.6)" | "rgba(255,255,255,0.60)" => "rgba(28, 28, 30, 0.65)".to_string(),
            _ => color_hex.to_string(),
        }
    } else {
        color_hex.to_string()
    };

    if let Some((dark_svg, light_svg)) = resolve_icon_pair(name) {
        let is_dark = is_dark_mode();
        let base_svg = if is_dark { dark_svg } else { light_svg };
        let colored_svg = base_svg
            .replace("#ffffff", &final_color)
            .replace("#FFFFFF", &final_color)
            .replace("#1c1c1e", &final_color)
            .replace("#1C1C1E", &final_color);
        return get_icon_from_svg(&colored_svg, size);
    }
    get_fallback_icon(name, "image-missing")
}

type IconWatchItem = (glib::WeakRef<gtk4::Image>, String, i32);

thread_local! {
    static ICON_WATCHERS: std::cell::RefCell<Vec<IconWatchItem>> = const { std::cell::RefCell::new(Vec::new()) };
    static WATCHER_REGISTERED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn register_icon_watcher(img: &gtk4::Image, name: &str, size: i32) {
    if let Some(settings) = gtk4::Settings::default() {
        WATCHER_REGISTERED.with(|registered| {
            if !registered.get() {
                settings.connect_gtk_application_prefer_dark_theme_notify(|_| {
                    ICON_WATCHERS.with(|watchers| {
                        let mut list = watchers.borrow_mut();
                        list.retain(|(weak_img, name, size)| {
                            if let Some(img) = weak_img.upgrade() {
                                load_icon_image_data(&img, name, *size);
                                true
                            } else {
                                false
                            }
                        });
                    });
                });
                registered.set(true);
            }
        });

        ICON_WATCHERS.with(|watchers| {
            watchers.borrow_mut().push((img.downgrade(), name.to_string(), size));
        });
    }
}

fn load_icon_image_data(img: &gtk4::Image, name: &str, size: i32) {
    if name == "logo" {
        let logo_img = get_logo_png(size);
        img.set_paintable(logo_img.paintable().as_ref());
        img.set_pixel_size(size);
        return;
    }
    let is_dark = is_dark_mode();

    if let Some((dark_svg, light_svg)) = resolve_icon_pair(name) {
        let svg_content = if is_dark { dark_svg } else { light_svg };
        let icon_img = get_icon_from_svg(svg_content, size);
        img.set_paintable(icon_img.paintable().as_ref());
        img.set_pixel_size(size);
    } else {
        let icon_img = get_fallback_icon(name, "image-missing");
        img.set_paintable(icon_img.paintable().as_ref());
        img.set_pixel_size(size);
    }
}

/// Sets the image content from local SVG or system icon theme.
pub fn set_image_from_icon(img: &gtk4::Image, name: &str, size: i32) {
    load_icon_image_data(img, name, size);
    ICON_WATCHERS.with(|watchers| {
        let mut list = watchers.borrow_mut();
        for (weak_img, watch_name, watch_size) in list.iter_mut() {
            if let Some(target) = weak_img.upgrade() {
                if target == *img {
                    *watch_name = name.to_string();
                    *watch_size = size;
                    return;
                }
            }
        }
    });
}

/// Helper function to retrieve an SVG icon widget by name. Defaults to white in dark mode and dark gray in light mode.
/// Automatically updates icon paintable when theme switches between Dark and Light mode.
pub fn get_icon(name: &str, size: i32) -> gtk4::Image {
    let img = gtk4::Image::new();
    load_icon_image_data(&img, name, size);
    register_icon_watcher(&img, name, size);
    img
}

/// Sets an icon on an existing image with fallback support.
pub fn set_fallback_icon(img: &gtk4::Image, icon_path_or_name: &str, default_fallback: &str) {
    if icon_path_or_name.is_empty() {
        img.set_icon_name(Some(default_fallback));
        return;
    }

    if icon_path_or_name.starts_with('/') && std::path::Path::new(icon_path_or_name).is_file() {
        img.set_from_file(Some(icon_path_or_name));
        return;
    }

    let mut clean_name = icon_path_or_name.to_string();
    for ext in &[".png", ".svg", ".xpm", ".jpg", ".jpeg", ".gif"] {
        if clean_name.to_lowercase().ends_with(ext) {
            clean_name = clean_name[..clean_name.len() - ext.len()].to_string();
            break;
        }
    }

    if clean_name == "logo" || clean_name.starts_with("babydra") {
        if let Some(tex) = get_logo_paintable(32) {
            img.set_paintable(Some(&tex));
            return;
        }
    }

    if let Some((dark_svg, light_svg)) = resolve_icon_pair(&clean_name) {
        let is_dark = is_dark_mode();
        let svg = if is_dark { dark_svg } else { light_svg };
        let icon_img = get_icon_from_svg(svg, 16);
        img.set_paintable(icon_img.paintable().as_ref());
        return;
    }

    img.set_icon_name(Some(default_fallback));
}

/// Returns a GTK Image with fallback support.
pub fn get_fallback_icon(icon_path_or_name: &str, default_fallback: &str) -> gtk4::Image {
    let img = gtk4::Image::new();
    set_fallback_icon(&img, icon_path_or_name, default_fallback);
    img
}

/// Retrieves an icon by name, prioritizing the active system icon theme (GtkIconTheme).
pub fn get_theme_or_svg_icon(name: &str, size: i32) -> gtk4::Image {
    let path = std::path::Path::new(name);
    if path.is_absolute() && path.exists() {
        let img = gtk4::Image::from_file(path);
        img.set_pixel_size(size);
        return img;
    }

    let display = gtk4::gdk::Display::default();
    if let Some(ref disp) = display {
        let theme = gtk4::IconTheme::for_display(disp);
        if theme.has_icon(name) {
            let img = gtk4::Image::from_icon_name(name);
            img.set_pixel_size(size);
            return img;
        }
    }

    get_icon(name, size)
}
