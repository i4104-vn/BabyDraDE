//! Udev-based discovery of primary DRM GPUs.

use std::error::Error;

use smithay::{
    backend::{
        drm::{DrmNode, NodeType},
        session::libseat::LibSeatSession,
        udev::UdevBackend,
    },
};
use tracing::info;

use crate::adapters::compositor::CalloopData;

use super::{handler::try_init_gpu, models::GpuInitResult};

/// Scans udev for primary DRM GPUs and initializes the first usable one.
///
/// # Arguments
/// * `session` - LibSeat session used to open candidate device nodes.
/// * `seat_name` - Name of the seat whose udev devices are scanned.
/// * `data` - Compositor data used while constructing display surfaces.
///
/// # Returns
/// The initialized GPU state, or an error when no GPU could be activated.
pub(crate) fn scan_and_init_gpu(
    session: &mut LibSeatSession,
    seat_name: &str,
    data: &mut CalloopData,
) -> Result<GpuInitResult, Box<dyn Error>> {
    let udev_backend = UdevBackend::new(seat_name).map_err(|err| {
        std::io::Error::other(format!(
            "Failed to initialize udev scanner on seat '{seat_name}': {err}"
        ))
    })?;

    info!("Scanning DRM devices via udev...");
    let mut candidate_gpus = Vec::new();
    for (_dev_id, path) in udev_backend.device_list() {
        info!("Udev detected device: {:?}", path);
        if let Ok(node) = DrmNode::from_path(path) {
            info!("  -> DrmNode type: {:?}", node.ty());
            if node.ty() == NodeType::Primary {
                candidate_gpus.push(path.to_path_buf());
            }
        }
    }

    info!(
        "Found {} candidate primary DRM GPU(s): {:?}",
        candidate_gpus.len(),
        candidate_gpus
    );

    for primary_gpu_path in candidate_gpus {
        info!("Attempting initialization on GPU: {:?}", primary_gpu_path);

        if let Some((drm_device, drm_notifier, surfaces, renderer, gbm_device)) =
            try_init_gpu(&primary_gpu_path, session, data)
        {
            return Ok(GpuInitResult {
                drm_device,
                drm_notifier,
                surfaces,
                renderer,
                gbm_device,
                udev_backend,
            });
        }
    }

    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Could not activate any DRM GPU with connected displays. Check GPU drivers and monitor cables.",
    )))
}
