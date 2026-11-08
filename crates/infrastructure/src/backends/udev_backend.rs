use std::{
    cell::RefCell,
    error::Error,
    rc::Rc,
    time::Duration,
};

use presentation::DesktopTheme;
use smithay::{
    backend::{
        allocator::{
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
            Format, Fourcc,
        },
        drm::{
            compositor::{DrmCompositor, FrameFlags},
            exporter::gbm::GbmFramebufferExporter,
            DrmDevice, DrmDeviceFd, DrmEvent, DrmNode, NodeType,
        },
        egl::{EGLContext, EGLDisplay},
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        renderer::{glow::GlowRenderer, ImportDma},
        session::{libseat::LibSeatSession, Event as SessionEvent, Session},
        udev::{UdevBackend, UdevEvent},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{
        calloop::{
            timer::{TimeoutAction, Timer},
            EventLoop,
        },
        drm::control::ModeTypeFlags,
        input::Libinput,
        rustix::fs::OFlags,
    },
    utils::{DeviceFd, Scale, Transform},
};
use smithay_drm_extras::drm_scanner::{DrmScanEvent, DrmScanner, SimpleCrtcMapper};
use tracing::{error, info, warn};

use crate::{
    compositor::{BabydraState, CalloopData},
    rendering::{render_window_with_borders, BorderShader, CornerShader, Cursor, CustomRenderElements},
};

type UdevDrmCompositor = DrmCompositor<
    GbmAllocator<DrmDeviceFd>,
    GbmFramebufferExporter<DrmDeviceFd>,
    (),
    DrmDeviceFd,
>;

struct SurfaceData {
    compositor: UdevDrmCompositor,
    output: Output,
}

struct UdevState {
    surfaces: Vec<SurfaceData>,
    renderer: GlowRenderer,
    theme: DesktopTheme,
    cursor: Cursor,
    active: bool,
    _drm_device: DrmDevice,
    _gbm_device: GbmDevice<DrmDeviceFd>,
    _session: LibSeatSession,
}

pub(crate) fn init(
    event_loop: &mut EventLoop<CalloopData>,
    data: &mut CalloopData,
) -> Result<(), Box<dyn Error>> {
    info!("Initializing BabyDraDE Udev/DRM backend for bare TTY/hardware scanout...");

    // 1. Claim seat via LibSeat (logind / seatd)
    let (mut session, notifier) = LibSeatSession::new().map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!(
                "Failed to claim seat session via LibSeat: {err}.\n\
                 Ensure systemd-logind or seatd is running and current user is in the 'seat' / 'video' / 'input' group."
            ),
        )
    })?;

    let seat_name = session.seat();
    info!("LibSeat session established on seat '{}'", seat_name);

    // 2. Setup Libinput for keyboard, pointer, and touch inputs on bare TTY
    let mut libinput_context = Libinput::new_with_udev(
        LibinputSessionInterface::from(session.clone()),
    );
    libinput_context
        .udev_assign_seat(&seat_name)
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to assign seat '{seat_name}' to Libinput"),
            )
        })?;
    let libinput_backend = LibinputInputBackend::new(libinput_context);

    event_loop
        .handle()
        .insert_source(libinput_backend, move |event, _, data| {
            data.state.process_input_event(event);
        })
        .map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to insert Libinput source: {err}"),
            )
        })?;

    // 3. Scan for Primary DRM GPU using Udev
    let udev_backend = UdevBackend::new(&seat_name).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to initialize udev scanner on seat '{seat_name}': {err}"),
        )
    })?;

    info!("Scanning DRM devices via udev...");
    let mut candidate_gpus = Vec::new();
    for (_dev_id, path) in udev_backend.device_list() {
        info!("Udev detected device: {:?}", path);
        if let Ok(node) = DrmNode::from_path(&path) {
            info!("  -> DrmNode type: {:?}", node.ty());
            if node.ty() == NodeType::Primary {
                candidate_gpus.push(path.to_path_buf());
            }
        }
    }

    info!("Found {} candidate primary DRM GPU(s): {:?}", candidate_gpus.len(), candidate_gpus);

    // Try each candidate GPU (e.g. card1, card0) until one successfully initializes with connectors
    let mut initialized_state: Option<(DrmDevice, smithay::backend::drm::DrmDeviceNotifier, Vec<SurfaceData>, GlowRenderer, GbmDevice<DrmDeviceFd>)> = None;

    for primary_gpu_path in &candidate_gpus {
        info!("Attempting initialization on GPU: {:?}", primary_gpu_path);

        let owned_fd = match session.open(
            primary_gpu_path,
            OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY,
        ) {
            Ok(fd) => fd,
            Err(err) => {
                warn!("Failed to open GPU {:?} via LibSeat: {err}. Trying next GPU...", primary_gpu_path);
                continue;
            }
        };

        let device_fd = DeviceFd::from(owned_fd);
        let drm_device_fd = DrmDeviceFd::new(device_fd);

        let (mut drm_device, drm_notifier) = match DrmDevice::new(drm_device_fd.clone(), false) {
            Ok(dev) => dev,
            Err(err) => {
                warn!("Failed to initialize DrmDevice on {:?}: {err}. Trying next GPU...", primary_gpu_path);
                continue;
            }
        };

        let gbm_device: GbmDevice<DrmDeviceFd> = match GbmDevice::new(drm_device_fd.clone()) {
            Ok(gbm) => gbm,
            Err(err) => {
                warn!("Failed to create GbmDevice on {:?}: {err}. Trying next GPU...", primary_gpu_path);
                continue;
            }
        };

        let allocator = GbmAllocator::new(gbm_device.clone(), GbmBufferFlags::RENDERING);
        let exporter = GbmFramebufferExporter::new(gbm_device.clone(), None);

        let egl_display = match unsafe { EGLDisplay::new(gbm_device.clone()) } {
            Ok(disp) => disp,
            Err(err) => {
                warn!("Failed to create EGLDisplay from GBM on {:?}: {err}. Trying next GPU...", primary_gpu_path);
                continue;
            }
        };

        let egl_context = match EGLContext::new(&egl_display) {
            Ok(ctx) => ctx,
            Err(err) => {
                warn!("Failed to create EGLContext on {:?}: {err}. Trying next GPU...", primary_gpu_path);
                continue;
            }
        };

        let mut renderer = match unsafe { GlowRenderer::new(egl_context) } {
            Ok(ren) => ren,
            Err(err) => {
                warn!("Failed to initialize GlowRenderer on {:?}: {err}. Trying next GPU...", primary_gpu_path);
                continue;
            }
        };

        BorderShader::init(&mut renderer);
        CornerShader::init(&mut renderer);

        let renderer_formats: Vec<Format> = <GlowRenderer as ImportDma>::dmabuf_formats(&renderer)
            .into_iter()
            .collect();

        let mut scanner: DrmScanner<SimpleCrtcMapper> = DrmScanner::new();
        let scan_result = match scanner.scan_connectors(&drm_device) {
            Ok(res) => res,
            Err(err) => {
                warn!("Failed to scan connectors on {:?}: {err}. Trying next GPU...", primary_gpu_path);
                continue;
            }
        };

        let mut surfaces: Vec<SurfaceData> = Vec::new();
        for event in scan_result {
            if let DrmScanEvent::Connected { connector, crtc: Some(crtc) } = event {
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

                    let conn_name = format!("{}-{}", connector.interface().as_str(), connector.interface_id());
                    info!(
                        "Successfully attached display output '{}' with mode {}x{}@{}Hz on GPU {:?}",
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
                    output.change_current_state(
                        Some(smithay_mode),
                        Some(Transform::Normal),
                        None,
                        Some((0, 0).into()),
                    );
                    output.set_preferred(smithay_mode);
                    data.state.space.map_output(&output, (0, 0));

                    let planes = drm_device.planes(&crtc).ok();
                    match drm_device.create_surface(crtc, drm_mode, &[connector.handle()]) {
                        Ok(drm_surface) => {
                            match DrmCompositor::new(
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
                                Ok(compositor) => {
                                    surfaces.push(SurfaceData { compositor, output });
                                }
                                Err(err) => {
                                    warn!("Failed to create DrmCompositor for connector on {:?}: {err}", primary_gpu_path);
                                }
                            }
                        }
                        Err(err) => {
                            warn!("Failed to create DRM surface for connector on {:?}: {err}", primary_gpu_path);
                        }
                    }
                }
            }
        }

        if !surfaces.is_empty() {
            info!("GPU {:?} successfully activated with {} display surface(s).", primary_gpu_path, surfaces.len());
            initialized_state = Some((drm_device, drm_notifier, surfaces, renderer, gbm_device));
            break;
        } else {
            warn!("GPU {:?} has no active display surfaces. Checking other GPUs...", primary_gpu_path);
        }
    }

    let (drm_device, drm_notifier, surfaces, renderer, gbm_device) = initialized_state.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not activate any DRM GPU with connected displays. Check GPU drivers and monitor cables.",
        )
    })?;

    let theme = DesktopTheme::magma_cyan();

    std::env::set_var("WAYLAND_DISPLAY", &data.state.socket_name);

    let state = Rc::new(RefCell::new(UdevState {
        surfaces,
        renderer,
        theme,
        cursor: Cursor::new(),
        active: true,
        _drm_device: drm_device,
        _gbm_device: gbm_device,
        _session: session,
    }));

    // 7. Insert Session Notifier (pause/resume when switching VTs with Ctrl+Alt+Fn)
    let state_session = state.clone();
    event_loop
        .handle()
        .insert_source(notifier, move |event, _, _data| {
            let mut s = state_session.borrow_mut();
            match event {
                SessionEvent::PauseSession => {
                    info!("Session paused (switched away to another VT). Suspending compositor render loop.");
                    s.active = false;
                }
                SessionEvent::ActivateSession => {
                    info!("Session activated (switched back to BabyDraDE VT). Resuming scanout.");
                    s.active = true;
                }
            }
        })
        .map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to insert session notifier: {err}"),
            )
        })?;

    // 8. Insert DRM event notifier (page-flip / vblank completion)
    let state_drm = state.clone();
    event_loop
        .handle()
        .insert_source(drm_notifier, move |event, _, _data| {
            let mut s = state_drm.borrow_mut();
            match event {
                DrmEvent::VBlank(_crtc) => {
                    for surface_data in s.surfaces.iter_mut() {
                        let _ = surface_data.compositor.frame_submitted();
                    }
                }
                DrmEvent::Error(err) => {
                    error!("DRM device event error: {err}");
                }
            }
        })
        .map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to insert DRM notifier: {err}"),
            )
        })?;

    // 9. Insert Udev event monitor for device hotplugging
    event_loop
        .handle()
        .insert_source(udev_backend, move |event, _, _data| {
            match event {
                UdevEvent::Added { device_id, path } => {
                    info!("GPU device added: dev_id={device_id}, path={path:?}");
                }
                UdevEvent::Changed { device_id } => {
                    info!("DRM device changed (possible monitor hotplug): dev_id={device_id}");
                }
                UdevEvent::Removed { device_id } => {
                    warn!("GPU device removed: dev_id={device_id}");
                }
            }
        })
        .map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to insert udev event source: {err}"),
            )
        })?;

    // 10. Frame Render Loop — driven by 16ms timer (~60 FPS)
    let state_render = state.clone();
    let timer = Timer::immediate();
    event_loop
        .handle()
        .insert_source(timer, move |_deadline, _, data| {
            let mut s = state_render.borrow_mut();
            if s.active {
                let state = &mut data.state;
                let display_handle = &mut data.display_handle;
                let scale = Scale::from(1.0);

                let UdevState {
                    ref mut surfaces,
                    ref mut renderer,
                    theme,
                    ref cursor,
                    ..
                } = *s;

                for surface_data in surfaces.iter_mut() {
                    let mut render_elements: Vec<CustomRenderElements> = Vec::new();

                    // 1. Mouse cursor MUST BE FIRST in front-to-back order (index 0 is on top of everything)
                    if let Some(cursor_elem) = cursor.element(renderer, state.pointer_location, scale) {
                        render_elements.push(cursor_elem);
                    }

                    // 2. Render each mapped window in front-to-back order (highest z-index / focused window first)
                    for window in state.space.elements().rev() {
                        if let Some(loc) = state.space.element_location(window) {
                            render_elements.extend(render_window_with_borders(
                                renderer,
                                window,
                                loc,
                                scale,
                                &theme.border,
                            ));
                        }
                    }

                    match surface_data.compositor.render_frame::<GlowRenderer, CustomRenderElements>(
                        renderer,
                        &render_elements,
                        theme.background_color,
                        FrameFlags::DEFAULT,
                    ) {
                        Ok(render_result) => {
                            if !render_result.is_empty {
                                if let Err(err) = surface_data.compositor.queue_frame(()) {
                                    warn!("Failed to queue DRM frame: {err}");
                                }
                            }
                        }
                        Err(err) => {
                            warn!("Failed to render DRM frame: {err}");
                        }
                    }

                    BorderShader::cleanup(renderer);

                    let output = &surface_data.output;
                    state.space.elements().for_each(|window| {
                        window.send_frame(
                            output,
                            state.start_time.elapsed(),
                            Some(Duration::ZERO),
                            |_, _| Some(output.clone()),
                        );
                    });
                }

                state.space.refresh();
                state.popups.cleanup();
                let _ = display_handle.flush_clients();
            }

            TimeoutAction::ToDuration(Duration::from_millis(16))
        })
        .map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to insert frame timer: {err}"),
            )
        })?;

    info!("BabyDraDE Udev/DRM backend initialized successfully. Hardware scanout active.");
    Ok(())
}
