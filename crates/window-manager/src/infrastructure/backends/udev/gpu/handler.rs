//! Single-GPU activation orchestration.

use std::path::PathBuf;

use smithay::backend::session::libseat::LibSeatSession;
use tracing::info;

use crate::adapters::compositor::CalloopData;

use super::{
    device::open_gpu,
    models::{GpuActivation, GpuContext},
    surfaces::build_surfaces,
};

/// Attempts to activate a GPU, returning its rendering state on success.
///
/// # Arguments
/// * `path` - Path of the DRM device node to activate.
/// * `session` - LibSeat session used to open the device node.
/// * `data` - Compositor data used while constructing display surfaces.
///
/// # Returns
/// `Some(GpuActivation)` when at least one display surface was created, `None` otherwise.
pub(super) fn try_init_gpu(
    path: &PathBuf,
    session: &mut LibSeatSession,
    data: &mut CalloopData,
) -> Option<GpuActivation> {
    let mut gpu: GpuContext = open_gpu(path, session)?;

    let surfaces = build_surfaces(&mut gpu, data);
    if surfaces.is_empty() {
        gpu.drm_device.pause();
        return None;
    }

    info!(
        "GPU {:?} successfully activated with {} display surface(s).",
        path,
        surfaces.len()
    );

    let GpuContext {
        drm_device,
        drm_notifier,
        gbm_device,
        renderer,
        ..
    } = gpu;

    Some((drm_device, drm_notifier, surfaces, renderer, gbm_device))
}
