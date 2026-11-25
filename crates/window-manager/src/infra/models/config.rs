//! Backend configuration models: GPU discovery results and frame contexts.

use std::cell::RefMut;

use smithay::{
    backend::{
        allocator::gbm::GbmDevice,
        drm::{DrmDevice, DrmDeviceFd, DrmDeviceNotifier},
        renderer::glow::GlowRenderer,
        udev::UdevBackend,
    },
};

use crate::{app::state::CalloopData, modules::shell::SwitcherOverlay};

use super::device::{SurfaceData, UdevState};

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
pub(crate) type GpuActivation = (
    DrmDevice,
    DrmDeviceNotifier,
    Vec<SurfaceData>,
    GlowRenderer,
    GbmDevice<DrmDeviceFd>,
);

/// Borrowed state required to render a single udev frame.
pub(crate) struct FrameRenderContext<'a> {
    pub(crate) data: &'a mut CalloopData,
    pub(crate) udev: RefMut<'a, UdevState>,
    pub(crate) switcher: &'a mut SwitcherOverlay,
}