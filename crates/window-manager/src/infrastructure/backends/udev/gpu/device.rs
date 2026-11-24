//! GPU device opening, DRM/GBM setup, EGL and renderer creation.

use std::path::PathBuf;

use smithay::{
    backend::{
        allocator::{
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
            Format,
        },
        drm::{exporter::gbm::GbmFramebufferExporter, DrmDevice, DrmDeviceFd},
        egl::{EGLContext, EGLDisplay},
        renderer::{glow::GlowRenderer, ImportDma},
        session::{libseat::LibSeatSession, Session},
    },
    reexports::rustix::fs::OFlags,
    utils::DeviceFd,
};
use tracing::warn;

use super::models::GpuContext;

/// Opens a GPU device and builds the DRM/GBM/EGL state used for rendering.
///
/// # Arguments
/// * `path` - Path of the DRM device node to open.
/// * `session` - LibSeat session used to gain access to the device node.
///
/// # Returns
/// `Some(GpuContext)` when every initialization stage succeeds, `None` otherwise.
pub(super) fn open_gpu(path: &PathBuf, session: &mut LibSeatSession) -> Option<GpuContext> {
    let owned_fd = match session.open(path, OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY) {
        Ok(fd) => fd,
        Err(err) => {
            warn!("Failed to open GPU {:?} via LibSeat: {err}", path);
            return None;
        }
    };

    let device_fd = DeviceFd::from(owned_fd);
    let drm_device_fd = DrmDeviceFd::new(device_fd);

    let (mut drm_device, drm_notifier) = match DrmDevice::new(drm_device_fd.clone(), false) {
        Ok(dev) => dev,
        Err(err) => {
            warn!("Failed to initialize DrmDevice on {:?}: {err}", path);
            return None;
        }
    };

    let gbm_device: GbmDevice<DrmDeviceFd> = match GbmDevice::new(drm_device_fd.clone()) {
        Ok(gbm) => gbm,
        Err(err) => {
            warn!("Failed to create GbmDevice on {:?}: {err}", path);
            drm_device.pause();
            return None;
        }
    };

    let allocator = GbmAllocator::new(gbm_device.clone(), GbmBufferFlags::RENDERING);
    let exporter = GbmFramebufferExporter::new(gbm_device.clone(), None);

    // SAFETY: Backend surface and renderer stay alive for this call.
    let egl_display = match unsafe { EGLDisplay::new(gbm_device.clone()) } {
        Ok(disp) => disp,
        Err(err) => {
            warn!("Failed to create EGLDisplay on {:?}: {err}", path);
            drm_device.pause();
            return None;
        }
    };

    let egl_context = match EGLContext::new(&egl_display) {
        Ok(ctx) => ctx,
        Err(err) => {
            warn!("Failed to create EGLContext on {:?}: {err}", path);
            drm_device.pause();
            return None;
        }
    };

    // SAFETY: Backend surface and renderer stay alive for this call.
    let renderer = match unsafe { GlowRenderer::new(egl_context) } {
        Ok(ren) => ren,
        Err(err) => {
            warn!("Failed to initialize GlowRenderer on {:?}: {err}", path);
            drm_device.pause();
            return None;
        }
    };

    let renderer_formats: Vec<Format> = <GlowRenderer as ImportDma>::dmabuf_formats(&renderer)
        .into_iter()
        .collect();

    Some(GpuContext {
        path: path.clone(),
        drm_device,
        drm_notifier,
        gbm_device,
        allocator,
        exporter,
        renderer,
        renderer_formats,
    })
}
