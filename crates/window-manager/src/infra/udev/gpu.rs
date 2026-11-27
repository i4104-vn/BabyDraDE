//! DRM/GPU discovery and initialization.
//!
//! GPU device opening, DRM/GBM setup, EGL and renderer creation.

use std::error::Error;
use std::path::Path;

use smithay::{
    backend::{
        allocator::{
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
            Format,
        },
        drm::{exporter::gbm::GbmFramebufferExporter, DrmDevice, DrmDeviceFd, DrmNode, NodeType},
        egl::{EGLContext, EGLDisplay},
        renderer::{glow::GlowRenderer, ImportDma},
        session::{libseat::LibSeatSession, Session},
        udev::UdevBackend,
    },
    reexports::rustix::fs::OFlags,
    utils::DeviceFd,
};
use tracing::{info, warn};

use crate::app::state::CalloopData;

use crate::infra::models::{GpuActivation, GpuContext, GpuInitResult, SurfaceData};

/// Opens a GPU device and builds the DRM/GBM/EGL state used for rendering.
///
/// # Arguments
/// * `path` - Path of the DRM device node to open.
/// * `session` - LibSeat session used to gain access to the device node.
///
/// # Returns
/// `Some(GpuContext)` when every initialization stage succeeds, `None` otherwise.
pub(crate) fn open_gpu(path: &Path, session: &mut LibSeatSession) -> Option<GpuContext> {
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

    // SAFETY: `gbm_device` is kept alive by the local binding and by
    // `GpuContext` for as long as the returned EGLDisplay uses it, and this
    // thread is the only one making the new context current here.
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

    // SAFETY: `egl_context` is moved into the renderer, which keeps it alive;
    // GL calls happen on the thread that owns the current EGL context.
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
        path: path.to_path_buf(),
        drm_device,
        drm_notifier,
        gbm_device,
        allocator,
        exporter,
        renderer,
        renderer_formats,
    })
}

/// Attempts to activate a GPU, returning its rendering state on success.
///
/// # Arguments
/// * `path` - Path of the DRM device node to activate.
/// * `session` - LibSeat session used to open the device node.
/// * `data` - Compositor data used while constructing display surfaces.
///
/// # Returns
/// `Some(GpuActivation)` when at least one display surface was created, `None` otherwise.
pub(crate) fn try_init_gpu(
    path: &Path,
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

use smithay::{
    backend::{allocator::Fourcc, drm::compositor::DrmCompositor},
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::drm::control::ModeTypeFlags,
};
use smithay_drm_extras::drm_scanner::{DrmScanEvent, DrmScanner, SimpleCrtcMapper};

use crate::app::state::BabydraState;

/// Scans connected outputs and builds a DRM surface for each usable connector.
///
/// # Arguments
/// * `gpu` - Initialized GPU state containing the device, allocator and renderer.
/// * `data` - Compositor data used to register Wayland outputs and map them into the space.
///
/// # Returns
/// The connected display surfaces; empty when scanning fails or no connector is usable.
pub(crate) fn build_surfaces(gpu: &mut GpuContext, data: &mut CalloopData) -> Vec<SurfaceData> {
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

                let transform = crate::infra::orientation::configured_transform(&conn_name)
                    .unwrap_or_else(|| {
                        crate::infra::orientation::landscape_fallback(smithay_mode.size)
                    });
                info!("Display '{}' initial transform: {transform:?}", conn_name);

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
