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
    rendering::{render_window_with_borders, BorderShader, CornerShader, CustomRenderElements},
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
    active: bool,
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

    let mut primary_gpu_path = None;
    for (_dev_id, path) in udev_backend.device_list() {
        if let Ok(node) = DrmNode::from_path(&path) {
            if node.ty() == NodeType::Primary {
                primary_gpu_path = Some(path);
                break;
            }
        }
    }

    let primary_gpu_path = primary_gpu_path.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No primary DRM GPU device node (/dev/dri/card*) found on this system.",
        )
    })?;

    info!("Selected primary GPU for scanout: {:?}", primary_gpu_path);

    // 4. Open DRM device via LibSeat session
    let owned_fd = session.open(
        &primary_gpu_path,
        OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY,
    )?;
    let device_fd = DeviceFd::from(owned_fd);
    let drm_device_fd = DrmDeviceFd::new(device_fd);

    let (mut drm_device, drm_notifier) = DrmDevice::new(drm_device_fd.clone(), false)?;
    let gbm_device: GbmDevice<DrmDeviceFd> = GbmDevice::new(drm_device_fd.clone())?;
    let allocator = GbmAllocator::new(gbm_device.clone(), GbmBufferFlags::RENDERING);
    let exporter = GbmFramebufferExporter::new(gbm_device.clone(), None);

    // 5. Initialize EGL & GlowRenderer directly from GBM native platform
    // This bypasses the nested EGL_EXT_platform_base requirement that crashed cage.
    let egl_display = unsafe { EGLDisplay::new(gbm_device.clone()) }?;
    let egl_context = EGLContext::new(&egl_display)?;
    let mut renderer = unsafe { GlowRenderer::new(egl_context) }?;

    // Compile MagmaWM-inspired GLSL shaders (neon glowing borders and rounded corners)
    BorderShader::init(&mut renderer);
    CornerShader::init(&mut renderer);

    let theme = DesktopTheme::magma_cyan();
    let renderer_formats: Vec<Format> = <GlowRenderer as ImportDma>::dmabuf_formats(&renderer)
        .into_iter()
        .collect();

    // 6. Scan connectors using DrmScanner & register Outputs
    let mut scanner: DrmScanner<SimpleCrtcMapper> = DrmScanner::new();
    let mut surfaces: Vec<SurfaceData> = Vec::new();

    for event in scanner.scan_connectors(&drm_device)? {
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
                    "Found active display output '{}' with mode {}x{}@{}Hz",
                    conn_name,
                    smithay_mode.size.w,
                    smithay_mode.size.h,
                    smithay_mode.refresh / 1000
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
                let drm_surface = drm_device.create_surface(crtc, drm_mode, &[connector.handle()])?;

                let compositor = DrmCompositor::new(
                    &output,
                    drm_surface,
                    planes,
                    allocator.clone(),
                    exporter.clone(),
                    [Fourcc::Argb8888, Fourcc::Xrgb8888],
                    renderer_formats.clone(),
                    (0, 0).into(),
                    Some(gbm_device.clone()),
                )?;

                surfaces.push(SurfaceData { compositor, output });
            }
        }
    }

    if surfaces.is_empty() {
        warn!("No active DRM connectors detected during initial scan. Running in headless mode until display connected.");
    }

    std::env::set_var("WAYLAND_DISPLAY", &data.state.socket_name);

    let state = Rc::new(RefCell::new(UdevState {
        surfaces,
        renderer,
        theme,
        active: true,
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
                    ..
                } = *s;

                for surface_data in surfaces.iter_mut() {
                    let mut render_elements: Vec<CustomRenderElements> = Vec::new();

                    for window in state.space.elements() {
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
