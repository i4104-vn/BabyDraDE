//! Bundled parameters for launcher surface rendering.

use smithay::backend::renderer::glow::GlowRenderer;
use smithay::utils::Scale;

use crate::modules::shell::launcher::search::SearchRegion;
use crate::modules::shell::launcher::BufferRegion;

/// Frame parameters shared by the launcher surface builders.
pub(super) struct SurfaceContext<'a> {
    pub renderer: &'a mut GlowRenderer,
    pub scale: Scale<f64>,
    pub screen: (i32, i32),
    pub logical: (f64, f64),
    pub search_presence: f64,
    pub dock_presence: f64,
    pub search_tick: u64,
    pub search_region: SearchRegion,
    pub region: BufferRegion,
}
