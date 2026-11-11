use services::theme::DesktopTheme;
use smithay::{
    backend::{
        allocator::gbm::{GbmAllocator, GbmDevice},
        drm::{
            compositor::DrmCompositor, exporter::gbm::GbmFramebufferExporter, DrmDevice,
            DrmDeviceFd,
        },
        renderer::glow::GlowRenderer,
        session::libseat::LibSeatSession,
    },
    output::Output,
};

use crate::rendering::Cursor;

pub type UdevDrmCompositor =
    DrmCompositor<GbmAllocator<DrmDeviceFd>, GbmFramebufferExporter<DrmDeviceFd>, (), DrmDeviceFd>;

pub struct SurfaceData {
    pub compositor: UdevDrmCompositor,
    pub output: Output,
}

pub struct UdevState {
    pub surfaces: Vec<SurfaceData>,
    pub renderer: GlowRenderer,
    pub theme: DesktopTheme,
    pub cursor: Cursor,
    pub active: bool,
    pub _drm_device: DrmDevice,
    pub _gbm_device: GbmDevice<DrmDeviceFd>,
    pub _session: LibSeatSession,
}

impl Drop for UdevState {
    fn drop(&mut self) {
        // Explicitly pause DRM device before dropping to avoid AtomicDrmDevice trying to
        // commit/restore previous state on an invalid or unprivileged FD on teardown.
        self._drm_device.pause();
    }
}
