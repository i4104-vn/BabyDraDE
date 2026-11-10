use std::{error::Error, path::PathBuf};

use smithay::{
    backend::{
        allocator::{
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
            Format, Fourcc,
        },
        drm::{
            compositor::DrmCompositor, exporter::gbm::GbmFramebufferExporter, DrmDevice,
            DrmDeviceFd, DrmNode, NodeType,
        },
        egl::{EGLContext, EGLDisplay},
        renderer::{glow::GlowRenderer, ImportDma},
        session::Session,
        udev::UdevBackend,
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{drm::control::ModeTypeFlags, rustix::fs::OFlags},
    utils::{DeviceFd, Transform},
};
use smithay_drm_extras::drm_scanner::{DrmScanEvent, DrmScanner, SimpleCrtcMapper};
use tracing::{info, warn};

use crate::compositor::{BabydraState, CalloopData};

use super::surface::SurfaceData;

pub struct GpuInitResult {
    pub drm_device: DrmDevice,
    pub drm_notifier: smithay::backend::drm::DrmDeviceNotifier,
    pub surfaces: Vec<SurfaceData>,
    pub renderer: GlowRenderer,
    pub gbm_device: GbmDevice<DrmDeviceFd>,
    pub udev_backend: UdevBackend,
}

type GpuActivation = (
    DrmDevice,
    smithay::backend::drm::DrmDeviceNotifier,
    Vec<SurfaceData>,
    GlowRenderer,
    GbmDevice<DrmDeviceFd>,
);

pub fn scan_and_init_gpu(
    session: &mut smithay::backend::session::libseat::LibSeatSession,
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

fn try_init_gpu(
    primary_gpu_path: &PathBuf,
    session: &mut smithay::backend::session::libseat::LibSeatSession,
    data: &mut CalloopData,
) -> Option<GpuActivation> {
    let owned_fd = match session.open(
        primary_gpu_path,
        OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY,
    ) {
        Ok(fd) => fd,
        Err(err) => {
            warn!("Failed to open GPU {:?} via LibSeat: {err}", primary_gpu_path);
            return None;
        }
    };

    let device_fd = DeviceFd::from(owned_fd);
    let drm_device_fd = DrmDeviceFd::new(device_fd);

    let (mut drm_device, drm_notifier) = match DrmDevice::new(drm_device_fd.clone(), false) {
        Ok(dev) => dev,
        Err(err) => {
            warn!("Failed to initialize DrmDevice on {:?}: {err}", primary_gpu_path);
            return None;
        }
    };

    let gbm_device: GbmDevice<DrmDeviceFd> = match GbmDevice::new(drm_device_fd.clone()) {
        Ok(gbm) => gbm,
        Err(err) => {
            warn!("Failed to create GbmDevice on {:?}: {err}", primary_gpu_path);
            drm_device.pause();
            return None;
        }
    };

    let allocator = GbmAllocator::new(gbm_device.clone(), GbmBufferFlags::RENDERING);
    let exporter = GbmFramebufferExporter::new(gbm_device.clone(), None);

    let egl_display = match unsafe { EGLDisplay::new(gbm_device.clone()) } {
        Ok(disp) => disp,
        Err(err) => {
            warn!("Failed to create EGLDisplay on {:?}: {err}", primary_gpu_path);
            drm_device.pause();
            return None;
        }
    };

    let egl_context = match EGLContext::new(&egl_display) {
        Ok(ctx) => ctx,
        Err(err) => {
            warn!("Failed to create EGLContext on {:?}: {err}", primary_gpu_path);
            drm_device.pause();
            return None;
        }
    };

    let renderer = match unsafe { GlowRenderer::new(egl_context) } {
        Ok(ren) => ren,
        Err(err) => {
            warn!("Failed to initialize GlowRenderer on {:?}: {err}", primary_gpu_path);
            drm_device.pause();
            return None;
        }
    };

    let renderer_formats: Vec<Format> = <GlowRenderer as ImportDma>::dmabuf_formats(&renderer)
        .into_iter()
        .collect();

    let mut scanner: DrmScanner<SimpleCrtcMapper> = DrmScanner::new();
    let scan_result = match scanner.scan_connectors(&drm_device) {
        Ok(res) => res,
        Err(err) => {
            warn!("Failed to scan connectors on {:?}: {err}", primary_gpu_path);
            drm_device.pause();
            return None;
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
                    primary_gpu_path
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
                data.state.space.map_output(&output, (0, 0));

                let planes = drm_device.planes(&crtc).ok();
                if let Ok(drm_surface) = drm_device.create_surface(crtc, drm_mode, &[connector.handle()]) {
                    if let Ok(compositor) = DrmCompositor::new(
                        &output,
                        drm_surface,
                        planes,
                        allocator.clone(),
                        exporter.clone(),
                        [Fourcc::Argb8888, Fourcc::Xrgb8888],
                        renderer_formats.clone(),
                        (0, 0).into(),
                        Some(gbm_device.clone()),
                    ) {
                        surfaces.push(SurfaceData { compositor, output });
                    }
                }
            }
        }
    }

    match surfaces.is_empty() {
        false => {
            info!(
                "GPU {:?} successfully activated with {} display surface(s).",
                primary_gpu_path,
                surfaces.len()
            );
            Some((
                drm_device,
                drm_notifier,
                surfaces,
                renderer,
                gbm_device,
            ))
        }
        true => {
            drm_device.pause();
            None
        }
    }
}
