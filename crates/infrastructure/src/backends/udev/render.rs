use std::{cell::RefCell, error::Error, rc::Rc, time::Duration};

use smithay::{
    backend::{
        drm::{compositor::FrameFlags, DrmEvent},
        renderer::glow::GlowRenderer,
        session::Event as SessionEvent,
        udev::UdevEvent,
    },
    reexports::calloop::{
        timer::{TimeoutAction, Timer},
        EventLoop,
    },
    utils::Scale,
};
use tracing::{error, info, warn};

use crate::{
    compositor::CalloopData,
    rendering::{render_layer_surfaces, render_window, CustomRenderElements},
};

use super::surface::UdevState;

pub fn setup_render_loop(
    event_loop: &mut EventLoop<CalloopData>,
    state: Rc<RefCell<UdevState>>,
    notifier: smithay::backend::session::libseat::LibSeatSessionNotifier,
    drm_notifier: smithay::backend::drm::DrmDeviceNotifier,
    udev_backend: smithay::backend::udev::UdevBackend,
) -> Result<(), Box<dyn Error>> {
    // 1. Session Notifier (pause/resume when switching VTs with Ctrl+Alt+Fn)
    let state_session = state.clone();
    event_loop
        .handle()
        .insert_source(notifier, move |event, _, _data| {
            let mut s = state_session.borrow_mut();
            match event {
                SessionEvent::PauseSession => {
                    info!("Session paused (switched away to another VT). Suspending scanout.");
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

    // 2. DRM event notifier (page-flip / vblank completion)
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

    // 3. Udev event monitor for device hotplugging
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

    // 4. Frame Render Loop — driven by 16ms timer (~60 FPS)
    let state_render = state;
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

                    let output = &surface_data.output;

                    // 2. Render Overlay & Top layer surfaces (topbar, dynamic island, notification popups)
                    render_elements.extend(render_layer_surfaces(
                        renderer,
                        output,
                        scale,
                        &[smithay::wayland::shell::wlr_layer::Layer::Overlay, smithay::wayland::shell::wlr_layer::Layer::Top],
                    ));

                    // 3. Render each mapped window in front-to-back order (highest z-index / focused window first)
                    for window in state.space.elements().rev() {
                        if let Some(loc) = state.space.element_location(window) {
                            render_elements.extend(render_window(
                                renderer,
                                window,
                                loc,
                                scale,
                            ));
                        }
                    }

                    // 4. Render Bottom & Background layer surfaces (wallpaper, desktop background)
                    render_elements.extend(render_layer_surfaces(
                        renderer,
                        output,
                        scale,
                        &[smithay::wayland::shell::wlr_layer::Layer::Bottom, smithay::wayland::shell::wlr_layer::Layer::Background],
                    ));

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

                    let output = &surface_data.output;
                    state.space.elements().for_each(|window| {
                        window.send_frame(
                            output,
                            state.start_time.elapsed(),
                            Some(Duration::ZERO),
                            |_, _| Some(output.clone()),
                        );
                    });

                    let layer_map = smithay::desktop::layer_map_for_output(output);
                    for layer in layer_map.layers() {
                        layer.send_frame(
                            output,
                            state.start_time.elapsed(),
                            Some(Duration::ZERO),
                            |_, _| Some(output.clone()),
                        );
                    }
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

    Ok(())
}

