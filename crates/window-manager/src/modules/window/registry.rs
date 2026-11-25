//! Z-ordered registry of managed windows.


pub(crate) use crate::modules::window::models::registry::WindowRegistry;

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::desktop::{Space, Window};

    #[test]
    fn active_window_is_none_for_an_empty_registry_and_space() {
        let registry = WindowRegistry::new(|_| false);
        let space: Space<Window> = Space::default();
        assert!(registry.active_window(&space).is_none());
    }
}
// Inlined from window/metadata.rs.
pub(crate) mod metadata {
    //! Extracts and caches window identity from Wayland protocol state.
    //!
    //! This cache lives in the adapter layer, not in `domain`: it is process state
    //! with a lifecycle, not a pure rule. Keeping it here lets both the compositor
    //! and the renderer query metadata without importing each other.

    use std::{
        collections::HashMap,
        sync::{OnceLock, RwLock},
    };

    use smithay::{
        desktop::Window,
        reexports::wayland_server::Resource,
        wayland::{compositor::with_states, shell::xdg::XdgToplevelSurfaceData},
    };

    use crate::shared::types::window::WindowMetadata;

    #[derive(Default)]
    struct MetadataCache(HashMap<String, WindowMetadata>);

    static CACHE: OnceLock<RwLock<MetadataCache>> = OnceLock::new();

    fn cache() -> &'static RwLock<MetadataCache> {
        CACHE.get_or_init(|| RwLock::new(MetadataCache::default()))
    }

    fn key(window: &Window) -> Option<String> {
        Some(format!("{:?}", window.toplevel()?.wl_surface().id()))
    }

    /// Extracts and caches a window's current identity from protocol state.
    pub(crate) fn refresh(window: &Window) {
        let (Some(key), Some(toplevel)) = (key(window), window.toplevel()) else {
            return;
        };
        let (app_id, title) = with_states(toplevel.wl_surface(), |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .and_then(|data| data.lock().ok())
                .map(|guard| {
                    (
                        guard.app_id.clone().unwrap_or_default(),
                        guard.title.clone().unwrap_or_default(),
                    )
                })
        })
        .unwrap_or_default();

        let app_name = services::system::window::get_app_display_name(&app_id, &title);
        if let Ok(mut cache) = cache().write() {
            cache.0.insert(
                key,
                WindowMetadata {
                    app_id,
                    title,
                    app_name,
                },
            );
        }
    }

    /// Returns cached metadata for a window, extracting it on first use.
    pub(crate) fn get(window: &Window) -> Option<WindowMetadata> {
        let key = key(window)?;
        if let Some(metadata) = cache().read().ok()?.0.get(&key).cloned() {
            return Some(metadata);
        }
        refresh(window);
        cache().read().ok()?.0.get(&key).cloned()
    }

    /// Drops cached metadata for a destroyed window.
    pub(crate) fn remove(window: &Window) {
        if let Some(key) = key(window) {
            if let Ok(mut cache) = cache().write() {
                cache.0.remove(&key);
            }
        }
    }
}

// Inlined from window/cursor.rs.
pub(crate) mod cursor {
    //! Maps Wayland resize edges to Smithay cursor icons.
    //!
    //! This is protocol/toolkit knowledge and therefore belongs in an adapter,
    //! not in `domain`.

    use smithay::{
        input::pointer::CursorIcon,
        reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
    };

    /// Returns the cursor shape that matches the resize edge being dragged.
    pub(crate) fn resize_cursor_icon(edges: ResizeEdge) -> CursorIcon {
        use ResizeEdge::*;
        match edges {
            TopLeft => CursorIcon::NwResize,
            TopRight => CursorIcon::NeResize,
            BottomLeft => CursorIcon::SwResize,
            BottomRight => CursorIcon::SeResize,
            Top => CursorIcon::NResize,
            Bottom => CursorIcon::SResize,
            Left => CursorIcon::WResize,
            Right => CursorIcon::EResize,
            _ => CursorIcon::AllResize,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn edges_map_to_directional_icons() {
            assert_eq!(
                resize_cursor_icon(ResizeEdge::TopLeft),
                CursorIcon::NwResize
            );
            assert_eq!(resize_cursor_icon(ResizeEdge::Right), CursorIcon::EResize);
            assert_eq!(resize_cursor_icon(ResizeEdge::None), CursorIcon::AllResize);
        }
    }
}

// Inlined from window/cursor_impl/mod.rs.
pub(crate) mod cursor_impl {
    //! Cursor rendering and icon selection.

    pub(crate) mod data {
        pub(crate) const CURSOR_WIDTH: i32 = 16;
        pub(crate) const CURSOR_HEIGHT: i32 = 20;

        pub(crate) const CURSOR_PIXELS: &[u8; 16 * 20] = &[
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0,
            1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 2, 2, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 2, 1, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
    }

    pub(crate) mod handler {
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
    }

    pub(crate) mod models {
        use std::collections::HashMap;

        use smithay::{
            backend::renderer::element::memory::MemoryRenderBuffer,
            input::pointer::CursorIcon,
            utils::{Physical, Point},
        };

        /// Icons loaded up front so switching shapes never stalls a frame.
        pub(super) const PRELOADED_ICONS: &[CursorIcon] = &[
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
        pub(super) struct LoadedCursor {
            pub(super) buffer: MemoryRenderBuffer,
            /// Position of the pointer tip inside the image, in buffer pixels.
            pub(super) hotspot: Point<i32, Physical>,
        }

        /// Software cursor drawn by the compositor from a configurable theme.
        ///
        /// Images for every cursor role (arrow, text, wait, resize, ...) are decoded
        /// by `services` — Xcursor or PNG, with no GTK involved — and cached as
        /// [`MemoryRenderBuffer`]s. The active image follows the last cursor requested
        /// by a Wayland client, with compositor-side overrides while moving or
        /// resizing windows.
        pub(crate) struct Cursor {
            pub(super) theme: String,
            pub(super) size: u32,
            pub(super) hotspot_x: u32,
            pub(super) hotspot_y: u32,
            pub(super) cache: HashMap<CursorIcon, LoadedCursor>,
            pub(super) fallback: LoadedCursor,
            pub(super) current: CursorIcon,
        }

        impl Default for Cursor {
            fn default() -> Self {
                Self::new()
            }
        }
    }

    pub(crate) mod render {
        use smithay::{
            backend::renderer::{
                element::{memory::MemoryRenderBufferRenderElement, Kind},
                glow::GlowRenderer,
            },
            utils::{Logical, Point, Scale},
        };

        use crate::modules::render::CustomRenderElements;

        use super::models::Cursor;

        impl Cursor {
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
    }

    pub(crate) use models::Cursor;
}
