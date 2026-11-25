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
