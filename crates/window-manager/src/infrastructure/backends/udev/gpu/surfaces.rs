//! Connector scanning and DRM surface construction.

use smithay::{
    backend::{
        allocator::Fourcc,
        drm::compositor::DrmCompositor,
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::drm::control::ModeTypeFlags,
    utils::Transform,
};
use smithay_drm_extras::drm_scanner::{DrmScanEvent, DrmScanner, SimpleCrtcMapper};
use tracing::{info, warn};

use crate::adapters::compositor::{BabydraState, CalloopData};

use super::{super::surface::SurfaceData, models::GpuContext};

/// Scans connected outputs and builds a DRM surface for each usable connector.
///
/// # Arguments
/// * `gpu` - Initialized GPU state containing the device, allocator and renderer.
/// * `data` - Compositor data used to register Wayland outputs and map them into the space.
///
/// # Returns
/// The connected display surfaces; empty when scanning fails or no connector is usable.
pub(super) fn build_surfaces(gpu: &mut GpuContext, data: &mut CalloopData) -> Vec<SurfaceData> {
    let mut scanner: DrmScanner<SimpleCrtcMapper> = DrmScanner::new();
    let scan_result = match scanner.scan_connectors(&gpu.drm_device) {
        Ok(res) => res,
        Err(err) => {
            warn!("Failed to scan connectors on {:?}: {err}", gpu.path);
            return Vec::new();
        }
    };

    let mut surfaces: Vec<SurfaceData> = Vec::new();
    for event in scan_result {
        if let DrmScanEvent::Connected {
            connector,
            crtc: Some(crtc),
        } = event
        {
            let modes = connector.modes();
            if let Some(preferred_mode) = modes
                .iter()
                .find(|m| m.mode_type().contains(ModeTypeFlags::PREFERRED))
                .or_else(|| modes.first())
            {
                let drm_mode = *preferred_mode;
                let mode_size = drm_mode.size();
                let smithay_mode = Mode {
                    size: (mode_size.0 as i32, mode_size.1 as i32).into(),
                    refresh: (drm_mode.vrefresh() * 1000) as i32,
                };

                let conn_name = format!(
                    "{}-{}",
                    connector.interface().as_str(),
                    connector.interface_id()
                );
                info!(
                    "Attached display output '{}' ({}x{}@{}Hz) on GPU {:?}",
                    conn_name,
                    smithay_mode.size.w,
                    smithay_mode.size.h,
                    smithay_mode.refresh / 1000,
                    gpu.path
                );

                let output = Output::new(
                    conn_name,
                    PhysicalProperties {
                        size: (0, 0).into(),
                        subpixel: Subpixel::Unknown,
                        make: "BabyDraDE".into(),
                        model: "DRM-KMS".into(),
                    },
                );

                let _global = output.create_global::<BabydraState>(&data.display_handle);
                let transform = match std::env::var("BABYDRADE_TRANSFORM").as_deref() {
                    Ok("90") => Transform::_90,
                    Ok("180") => Transform::_180,
                    Ok("270") => Transform::_270,
                    Ok("flipped") => Transform::Flipped,
                    Ok("flipped-90") => Transform::Flipped90,
                    Ok("flipped-180") => Transform::Flipped180,
                    Ok("flipped-270") => Transform::Flipped270,
                    Ok("normal") | Ok("0") => Transform::Normal,
                    _ => {
                        let orientation = std::env::var("BABYDRADE_ORIENTATION")
                            .unwrap_or_else(|_| "landscape".to_string());
                        if orientation == "landscape" && smithay_mode.size.w < smithay_mode.size.h {
                            info!(
                                "Portrait display detected ({}x{}). Auto-rotating to landscape (Transform::_90)",
                                smithay_mode.size.w, smithay_mode.size.h
                            );
                            Transform::_90
                        } else {
                            Transform::Normal
                        }
                    }
                };

                output.change_current_state(
                    Some(smithay_mode),
                    Some(transform),
                    None,
                    Some((0, 0).into()),
                );
                output.set_preferred(smithay_mode);
                data.state.windows.space.map_output(&output, (0, 0));

                let planes = gpu.drm_device.planes(&crtc).ok();
                if let Ok(drm_surface) =
                    gpu.drm_device
                        .create_surface(crtc, drm_mode, &[connector.handle()])
                {
                    if let Ok(compositor) = DrmCompositor::new(
                        &output,
                        drm_surface,
                        planes,
                        gpu.allocator.clone(),
                        gpu.exporter.clone(),
                        [Fourcc::Argb8888, Fourcc::Xrgb8888],
                        gpu.renderer_formats.clone(),
                        (0, 0).into(),
                        Some(gpu.gbm_device.clone()),
                    ) {
                        surfaces.push(SurfaceData { compositor, output });
                    }
                }
            }
        }
    }

    surfaces
}
