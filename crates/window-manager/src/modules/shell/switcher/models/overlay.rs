//! Switcher overlay render-state model.

use std::time::Instant;

use smithay::backend::renderer::element::memory::MemoryRenderBuffer;

/// Cache key for the foreground buffer: `(start, selected, hovered, count, ...ticks)`.
pub(crate) type RenderKey = (Instant, usize, Option<usize>, usize, u64, u64, u64, u64);

#[derive(Default)]
pub(crate) struct SwitcherOverlay {
    pub(crate) fg_buffer: Option<MemoryRenderBuffer>,
    /// Physical region covered by `fg_buffer`: `(x, y, width, height)`.
    /// It only holds the dock rather than the whole screen, so each animation
    /// frame allocates and copies a few hundred KB instead of the full framebuffer.
    pub(crate) fg_region: Option<(i32, i32, i32, i32)>,
    pub(crate) last_rendered: Option<RenderKey>,
    pub(crate) backdrop_buffer: Option<MemoryRenderBuffer>,
    pub(crate) backdrop_size: Option<(i32, i32)>,
    pub(crate) backdrop_tick: Option<u64>,
}
