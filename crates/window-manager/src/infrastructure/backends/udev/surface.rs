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

use crate::adapters::compositor::Cursor;

pub(crate) type UdevDrmCompositor =
    DrmCompositor<GbmAllocator<DrmDeviceFd>, GbmFramebufferExporter<DrmDeviceFd>, (), DrmDeviceFd>;

pub(crate) struct SurfaceData {
    pub(crate) compositor: UdevDrmCompositor,
    pub(crate) output: Output,
}

pub(crate) struct UdevState {
    pub(crate) surfaces: Vec<SurfaceData>,
    pub(crate) renderer: GlowRenderer,
    pub(crate) theme: DesktopTheme,
    pub(crate) cursor: Cursor,
    pub(crate) active: bool,
    pub(crate) _drm_device: DrmDevice,
    pub(crate) _gbm_device: GbmDevice<DrmDeviceFd>,
    pub(crate) _session: LibSeatSession,
}

impl Drop for UdevState {
    fn drop(&mut self) {
        // Explicitly pause DRM device before dropping to avoid AtomicDrmDevice trying to
        // commit/restore previous state on an invalid or unprivileged FD on teardown.
        self._drm_device.pause();
    }
}
