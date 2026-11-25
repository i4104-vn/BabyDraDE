//! Udev backend event sources and frame rendering.
//!
//! Shared state for udev frame rendering.

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use animation::FRAME_INTERVAL;
use smithay::{
    backend::{
        drm::compositor::FrameFlags,
        renderer::glow::GlowRenderer,
    },
    reexports::calloop::{
        timer::{TimeoutAction, Timer},
        EventLoop,
    },
    utils::Scale,
};
use tracing::warn;

use crate::infra::models::{FrameRenderContext, UdevState};
use crate::{
    app::state::CalloopData,
    infra::udev::handler::{register_drm_notifier, register_session_notifier, register_udev_monitor},
    modules::{
        render::{compose_frame, CustomRenderElements, FrameContext as ShellFrameContext},
        shell::{
            screenshot::capture::{capture_screen, take_requested},
            SwitcherOverlay,
        },
    },
};


/// Registers the immediate timer that drives udev frame rendering.
///
/// # Arguments
/// * `event_loop` - The compositor event loop to attach the timer to.
/// * `state` - Shared udev state read and updated on every frame tick.
///
/// # Returns
/// `Ok(())` when the timer was inserted, otherwise an error describing the failure.
pub(crate) fn insert_frame_timer(
    event_loop: &mut EventLoop<'_, CalloopData>,
    state: Rc<RefCell<UdevState>>,
) -> Result<(), Box<dyn Error>> {
    let mut switcher = SwitcherOverlay::default();
    let timer = Timer::immediate();
    event_loop
        .handle()
        .insert_source(timer, move |deadline, _, data| {
            let ctx = FrameRenderContext {
                data,
                udev: state.borrow_mut(),
                switcher: &mut switcher,
            };
            render_frame(ctx);

            // Schedule from the previous deadline, not from render completion,
            // so a slow frame cannot permanently drift the 120 fps cadence.
            TimeoutAction::ToInstant(deadline + FRAME_INTERVAL)
        })
        .map_err(|err| std::io::Error::other(format!("Failed to insert frame timer: {err}")))?;

    Ok(())
}

/// Renders every active output for a single frame deadline.
pub(crate) fn render_frame(ctx: FrameRenderContext<'_>) {
    let FrameRenderContext {
        data,
        mut udev,
        switcher,
    } = ctx;
    if !udev.active {
        return;
    }

    let state = &mut data.state;
    crate::infra::frame_pacing::begin_frame(state);
    let display_handle = &mut data.display_handle;
    let scale = Scale::from(1.0);

    let UdevState {
        ref mut surfaces,
        ref mut renderer,
        ref theme,
        ref mut cursor,
        ..
    } = *udev;

    let mut screenshot_requested = take_requested();

    for surface_data in surfaces.iter_mut() {
        let output = &surface_data.output;
        let oriented_size = output
            .current_mode()
            .map(|mode| output.current_transform().transform_size(mode.size));

        cursor.set_icon(state.effective_cursor_icon());
        let cursor_element = if state.input.cursor_visible {
            cursor.element(renderer, state.input.pointer_location, scale)
        } else {
            None
        };

        let below_windows = oriented_size
            .map(|size| {
                state
                    .overlays
                    .launcher
                    .render(renderer, (size.w, size.h), scale)
            })
            .unwrap_or_default();
        let above_windows = oriented_size
            .map(|size| {
                switcher.render(
                    renderer,
                    state.overlays.alt_tab_switcher.as_ref(),
                    (size.w, size.h),
                    scale,
                )
            })
            .unwrap_or_default();

        let render_elements: Vec<CustomRenderElements> = {
            let frame_ctx = ShellFrameContext {
                output,
                scale,
                cursor: cursor_element,
                below_windows,
                above_windows,
                space: &state.windows.space,
                blur_engine: Some(&state.resources.render.blur_engine),
                pointer: state.input.pointer_location,
                visuals: &crate::modules::window::ops::window_animation_frame,
            };
            compose_frame(renderer, frame_ctx)
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
            let _ = capture_screen(renderer, &render_elements, output);
            screenshot_requested = false;
        }

        crate::infra::frame_pacing::send_frames(state, output);
    }

    crate::infra::frame_pacing::end_frame(state, display_handle);
}

/// Registers session, DRM, udev and frame event sources on the event loop.
///
/// # Arguments
/// * `event_loop` - The compositor event loop to attach sources to.
/// * `state` - Shared udev state updated by the session and DRM notifiers.
/// * `notifier` - LibSeat session notifier for VT switches.
/// * `drm_notifier` - DRM notifier for page-flip and vblank events.
/// * `udev_backend` - Udev monitor for device hotplugging.
///
/// # Returns
/// `Ok(())` when all sources were inserted, otherwise an error describing the failure.
pub(crate) fn setup_render_loop(
    event_loop: &mut EventLoop<'_, CalloopData>,
    state: Rc<RefCell<UdevState>>,
    notifier: smithay::backend::session::libseat::LibSeatSessionNotifier,
    drm_notifier: smithay::backend::drm::DrmDeviceNotifier,
    udev_backend: smithay::backend::udev::UdevBackend,
) -> Result<(), Box<dyn Error>> {
    register_session_notifier(event_loop, state.clone(), notifier)?;
    register_drm_notifier(event_loop, state.clone(), drm_notifier)?;
    register_udev_monitor(event_loop, udev_backend)?;

    // 4. Frame render loop — paced for 120 FPS via the shared frame budget.
    insert_frame_timer(event_loop, state)?;

    Ok(())
}
