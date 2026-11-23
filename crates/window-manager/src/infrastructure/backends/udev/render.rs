use std::{cell::RefCell, error::Error, rc::Rc};

use animation::FRAME_INTERVAL;
use smithay::{
    backend::{
        drm::{compositor::FrameFlags, DrmEvent},
        renderer::glow::GlowRenderer,
        udev::UdevEvent,
    },
    reexports::calloop::{
        timer::{TimeoutAction, Timer},
        EventLoop,
    },
    utils::Scale,
};
use tracing::{error, info, warn};

use super::surface::UdevState;
use crate::adapters::{
    compositor::CalloopData,
    desktop::SwitcherOverlay,
    rendering::{compose_frame, CustomRenderElements, FrameContext},
};

pub(crate) fn setup_render_loop(
    event_loop: &mut EventLoop<'_, CalloopData>,
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
                smithay::backend::session::Event::PauseSession => {
                    info!("Session paused (switched away to another VT). Suspending scanout.");
                    s.active = false;
                    s._drm_device.pause();
                }
                smithay::backend::session::Event::ActivateSession => {
                    info!("Session activated (switched back to BabyDraDE VT). Resuming scanout.");
                    s.active = true;
                    if let Err(err) = s._drm_device.activate(false) {
                        warn!("Failed to reactivate DRM device: {err}");
                    }
                }
            }
        })
        .map_err(|err| {
            std::io::Error::other(format!("Failed to insert session notifier: {err}"))
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
        .map_err(|err| std::io::Error::other(format!("Failed to insert DRM notifier: {err}")))?;

    // 3. Udev event monitor for device hotplugging
    event_loop
        .handle()
        .insert_source(udev_backend, move |event, _, _data| match event {
            UdevEvent::Added { device_id, path } => {
                info!("GPU device added: dev_id={device_id}, path={path:?}");
            }
            UdevEvent::Changed { device_id } => {
                info!("DRM device changed (possible monitor hotplug): dev_id={device_id}");
            }
            UdevEvent::Removed { device_id } => {
                warn!("GPU device removed: dev_id={device_id}");
            }
        })
        .map_err(|err| {
            std::io::Error::other(format!("Failed to insert udev event source: {err}"))
        })?;

    // 4. Frame Render Loop — paced for 120 FPS via the shared frame budget.
    let state_render = state;
    let mut switcher_overlay = SwitcherOverlay::default();
    let timer = Timer::immediate();
    event_loop
        .handle()
        .insert_source(timer, move |deadline, _, data| {
            let mut s = state_render.borrow_mut();
            if s.active {
                let state = &mut data.state;
                crate::infrastructure::backends::render_loop::begin_frame(state);
                let display_handle = &mut data.display_handle;
                let scale = Scale::from(1.0);

                let UdevState {
                    ref mut surfaces,
                    ref mut renderer,
                    theme,
                    ref mut cursor,
                    ..
                } = *s;

                let mut screenshot_requested =
                    crate::adapters::desktop::screenshot::capture::take_requested();

                for surface_data in surfaces.iter_mut() {
                    let output = &surface_data.output;
                    let oriented_size = output
                        .current_mode()
                        .map(|mode| output.current_transform().transform_size(mode.size));

                    cursor.set_icon(state.effective_cursor_icon());
                    let cursor_element = if state.cursor_visible {
                        cursor.element(renderer, state.pointer_location, scale)
                    } else {
                        None
                    };

                    let below_windows = oriented_size
                        .map(|size| state.launcher.render(renderer, (size.w, size.h), scale))
                        .unwrap_or_default();
                    let above_windows = oriented_size
                        .map(|size| {
                            switcher_overlay.render(
                                renderer,
                                state.alt_tab_switcher.as_ref(),
                                (size.w, size.h),
                                scale,
                            )
                        })
                        .unwrap_or_default();

                    let render_elements: Vec<CustomRenderElements> = {
                        let ctx = FrameContext {
                            output,
                            scale,
                            cursor: cursor_element,
                            below_windows,
                            above_windows,
                            space: &state.space,
                            blur_engine: Some(&state.blur_engine),
                            pointer: state.pointer_location,
                            visuals: &crate::adapters::compositor::window_animation_frame,
                        };
                        compose_frame(renderer, ctx)
                    };

                    match surface_data
                        .compositor
                        .render_frame::<GlowRenderer, CustomRenderElements>(
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

                    if screenshot_requested {
                        let _ = crate::adapters::desktop::screenshot::capture::capture_screen(
                            renderer,
                            &render_elements,
                            output,
                        );
                        screenshot_requested = false;
                    }

                    crate::infrastructure::backends::render_loop::send_frames(state, output);
                }

                crate::infrastructure::backends::render_loop::end_frame(state, display_handle);
            }

            // Schedule from the previous deadline, not from render completion,
            // so a slow frame cannot permanently drift the 120 fps cadence.
            TimeoutAction::ToInstant(deadline + FRAME_INTERVAL)
        })
        .map_err(|err| std::io::Error::other(format!("Failed to insert frame timer: {err}")))?;

    Ok(())
}
