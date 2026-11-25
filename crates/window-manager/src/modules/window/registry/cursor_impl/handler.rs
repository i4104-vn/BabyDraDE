use std::collections::HashMap;

use smithay::{
    backend::{allocator::Fourcc, renderer::element::memory::MemoryRenderBuffer},
    input::pointer::CursorIcon,
    utils::{Buffer, Point, Rectangle, Transform},
};

use services::theme::cursor::{load_cursor_icon, CursorImage};

use super::data::{CURSOR_HEIGHT, CURSOR_PIXELS, CURSOR_WIDTH};
use super::models::{Cursor, LoadedCursor, PRELOADED_ICONS};

impl Cursor {
    /// Builds the cursor cache from the theme selected in `configs/cursor.toml`.
    ///
    /// Individual roles missing from the theme fall back to the built-in
    /// pixel arrow, so partial themes still work.
    pub(crate) fn new() -> Self {
        let config = services::config::load_cursor_config();
        let mut cache = HashMap::new();
        for &icon in PRELOADED_ICONS {
            if let Some(image) = load_cursor_icon(
                &config.theme,
                icon,
                config.size,
                config.hotspot_x,
                config.hotspot_y,
            ) {
                cache.insert(icon, Self::upload(image));
            }
        }

        Self {
            theme: config.theme,
            size: config.size,
            hotspot_x: config.hotspot_x,
            hotspot_y: config.hotspot_y,
            fallback: Self::upload(Self::builtin_image()),
            cache,
            current: CursorIcon::Default,
        }
    }

    /// The fallback arrow shipped with the compositor.
    fn builtin_image() -> CursorImage {
        let mut rgba = Vec::with_capacity(CURSOR_PIXELS.len() * 4);
        for &pixel_type in CURSOR_PIXELS {
            let pixel = match pixel_type {
                1 => [0, 0, 0, 255],       // Black outline
                2 => [230, 248, 255, 255], // Bright cyan-white body
                _ => [0, 0, 0, 0],         // Transparent
            };
            rgba.extend_from_slice(&pixel);
        }

        CursorImage::from_rgba(CURSOR_WIDTH as u32, CURSOR_HEIGHT as u32, 0, 0, &rgba)
            .expect("built-in cursor dimensions and pixel data must match")
    }

    fn upload(image: CursorImage) -> LoadedCursor {
        let width = image.width as i32;
        let height = image.height as i32;
        let mut buffer = MemoryRenderBuffer::new(
            Fourcc::Argb8888,
            (width, height),
            1,
            Transform::Normal,
            None,
        );
        {
            let mut context = buffer.render();
            let pixels = &image.pixels;
            let _ = context.draw(|buf| {
                if buf.len() == pixels.len() {
                    buf.copy_from_slice(pixels);
                }
                Ok::<_, ()>(vec![Rectangle::<i32, Buffer>::new(
                    (0, 0).into(),
                    (width, height).into(),
                )])
            });
        }

        LoadedCursor {
            buffer,
            hotspot: Point::from((image.xhot as i32, image.yhot as i32)),
        }
    }

    /// Lazily loads roles outside the preloaded set (e.g. a client requesting
    /// an unusual icon for the first time).
    fn ensure_loaded(&mut self, icon: CursorIcon) {
        if self.cache.contains_key(&icon) {
            return;
        }
        if let Some(image) =
            load_cursor_icon(&self.theme, icon, self.size, self.hotspot_x, self.hotspot_y)
        {
            let loaded = Self::upload(image);
            self.cache.insert(icon, loaded);
        }
    }

    /// Switches the active cursor shape.
    pub(crate) fn set_icon(&mut self, icon: CursorIcon) {
        if self.current == icon {
            return;
        }
        self.current = icon;
        self.ensure_loaded(icon);
    }
}
