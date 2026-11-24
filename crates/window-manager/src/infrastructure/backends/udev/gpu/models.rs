//! GPU initialization result models.

use std::path::PathBuf;

use smithay::backend::{
    allocator::gbm::{GbmDevice},
    drm::{DrmDevice, DrmDeviceFd},
    renderer::glow::GlowRenderer,
    udev::UdevBackend,
};

use super::super::surface::SurfaceData;

pub(crate) struct GpuInitResult {
    pub(crate) drm_device: DrmDevice,
    pub(crate) drm_notifier: smithay::backend::drm::DrmDeviceNotifier,
    pub(crate) surfaces: Vec<SurfaceData>,
    pub(crate) renderer: GlowRenderer,
    pub(crate) gbm_device: GbmDevice<DrmDeviceFd>,
    pub(crate) udev_backend: UdevBackend,
}

pub(super) type GpuActivation = (
    DrmDevice,
    smithay::backend::drm::DrmDeviceNotifier,
    Vec<SurfaceData>,
    GlowRenderer,
    GbmDevice<DrmDeviceFd>,
);

