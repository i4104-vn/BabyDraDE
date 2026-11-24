//! GPU initialization result models.

use std::path::PathBuf;

use smithay::backend::{
    allocator::{
        gbm::{GbmAllocator, GbmDevice},
        Format,
    },
    drm::{exporter::gbm::GbmFramebufferExporter, DrmDevice, DrmDeviceFd, DrmDeviceNotifier},
    renderer::glow::GlowRenderer,
    udev::UdevBackend,
};

use super::super::surface::SurfaceData;

/// Fully initialized backend state returned by GPU discovery.
pub(crate) struct GpuInitResult {
    pub(crate) drm_device: DrmDevice,
    pub(crate) drm_notifier: DrmDeviceNotifier,
    pub(crate) surfaces: Vec<SurfaceData>,
    pub(crate) renderer: GlowRenderer,
    pub(crate) gbm_device: GbmDevice<DrmDeviceFd>,
    pub(crate) udev_backend: UdevBackend,
}

/// DRM device state returned by a successful single-GPU activation.
pub(super) type GpuActivation = (
    DrmDevice,
    DrmDeviceNotifier,
    Vec<SurfaceData>,
    GlowRenderer,
    GbmDevice<DrmDeviceFd>,
);

/// State shared across the GPU initialization stages.
pub(super) struct GpuContext {
    pub(super) path: PathBuf,
    pub(super) drm_device: DrmDevice,
    pub(super) drm_notifier: DrmDeviceNotifier,
    pub(super) gbm_device: GbmDevice<DrmDeviceFd>,
    pub(super) allocator: GbmAllocator<DrmDeviceFd>,
    pub(super) exporter: GbmFramebufferExporter<DrmDeviceFd>,
    pub(super) renderer: GlowRenderer,
    pub(super) renderer_formats: Vec<Format>,
}
