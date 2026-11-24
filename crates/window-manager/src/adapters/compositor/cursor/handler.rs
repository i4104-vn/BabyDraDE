use std::collections::HashMap;

use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            element::{
                memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
                Kind,
            },
            glow::GlowRenderer,
        },
    },
    input::pointer::CursorIcon,
    utils::{Buffer, Logical, Physical, Point, Rectangle, Scale, Transform},
};

use services::theme::cursor::{load_cursor_icon, CursorImage};

use crate::adapters::rendering::CustomRenderElements;
use super::data::{CURSOR_HEIGHT, CURSOR_PIXELS, CURSOR_WIDTH};

/// Icons loaded up front so switching shapes never stalls a frame.
const PRELOADED_ICONS: &[CursorIcon] = &[
    CursorIcon::Default,
    CursorIcon::ContextMenu,
    CursorIcon::Help,
    CursorIcon::Pointer,
    CursorIcon::Progress,
    CursorIcon::Wait,
    CursorIcon::Cell,
    CursorIcon::Crosshair,
    CursorIcon::Text,
    CursorIcon::VerticalText,
    CursorIcon::Alias,
    CursorIcon::Copy,
    CursorIcon::Move,
    CursorIcon::NoDrop,
    CursorIcon::NotAllowed,
    CursorIcon::Grab,
    CursorIcon::Grabbing,
    CursorIcon::EResize,
    CursorIcon::NResize,
    CursorIcon::NeResize,
    CursorIcon::NwResize,
    CursorIcon::SResize,
    CursorIcon::SeResize,
    CursorIcon::SwResize,
    CursorIcon::WResize,
    CursorIcon::EwResize,
    CursorIcon::NsResize,
    CursorIcon::NeswResize,
    CursorIcon::NwseResize,
    CursorIcon::ColResize,
    CursorIcon::RowResize,
    CursorIcon::AllScroll,
    CursorIcon::ZoomIn,
    CursorIcon::ZoomOut,
    CursorIcon::DndAsk,
    CursorIcon::AllResize,
];

/// A decoded theme image uploaded into a renderable memory buffer.
struct LoadedCursor {
    buffer: MemoryRenderBuffer,
    /// Position of the pointer tip inside the image, in buffer pixels.
    hotspot: Point<i32, Physical>,
}

/// Software cursor drawn by the compositor from a configurable theme.
///
/// Images for every cursor role (arrow, text, wait, resize, ...) are decoded
/// by `services` — Xcursor or PNG, with no GTK involved — and cached as
/// [`MemoryRenderBuffer`]s. The active image follows the last cursor requested
/// by a Wayland client, with compositor-side overrides while moving or
/// resizing windows.
pub(crate) struct Cursor {
    theme: String,
    size: u32,
    hotspot_x: u32,
    hotspot_y: u32,
    cache: HashMap<CursorIcon, LoadedCursor>,
    fallback: LoadedCursor,
    current: CursorIcon,
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

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

    pub(crate) fn element(
        &self,
        renderer: &mut GlowRenderer,
        pos: Point<f64, Logical>,
        scale: Scale<f64>,
    ) -> Option<CustomRenderElements> {
        let cursor = self.cache.get(&self.current).unwrap_or(&self.fallback);

        // Align the image's hotspot (not its top-left corner) with the pointer.
        let phys_pos = pos.to_physical_precise_round(scale) - cursor.hotspot;
        MemoryRenderBufferRenderElement::from_buffer(
            renderer,
            phys_pos.to_f64(),
            &cursor.buffer,
            None,
            None,
            None,
            Kind::Cursor,
        )
        .ok()
        .map(CustomRenderElements::Memory)
    }
}
