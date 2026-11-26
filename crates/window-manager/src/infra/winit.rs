use std::error::Error;

use services::theme::DesktopTheme;
use smithay::{
    backend::{
        renderer::{damage::OutputDamageTracker, glow::GlowRenderer},
        winit::{self, WinitEvent},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::calloop::EventLoop,
    utils::{Scale, Transform},
};

use crate::{
    app::state::{BabydraState, CalloopData},
    modules::render::{compose_frame, CustomRenderElements, FrameContext},
    modules::shell::SwitcherOverlay,
    modules::window::cursor_impl::Cursor,
};
const OUTPUT_REFRESH_MHZ: i32 = animation::TARGET_FPS as i32 * 1000;

pub(crate) fn init(
    event_loop: &mut EventLoop<'_, CalloopData>,
    data: &mut CalloopData,
) -> Result<(), Box<dyn Error>> {
    let (mut backend, winit) = winit::init::<GlowRenderer>().map_err(|err| {
        std::io::Error::other(format!(
            "Failed to initialize graphical window (Winit backend): {}",
            err
        ))
    })?;
    backend.window().set_title("BabyDraDE");
    // The compositor draws its own themed cursor into the frame; leaving the
    // host cursor visible would stack an arrow on top of it (and hide shape
    // changes) when running nested.
    backend.window().set_cursor_visible(false);

    let mut cursor = Cursor::new();
    let theme = DesktopTheme::default();

    let output = Output::new(
        "babydrade-0".to_string(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "BabyDraDE".into(),
            model: "Winit".into(),
        },
    );
    let mode = Mode {
        size: backend.window_size(),
        refresh: OUTPUT_REFRESH_MHZ,
    };
    let _output_global = output.create_global::<BabydraState>(&data.display_handle);
    let transform = crate::infra::orientation::primary_transform().unwrap_or(Transform::Normal);
    output.change_current_state(Some(mode), Some(transform), None, Some((0, 0).into()));
    output.set_preferred(mode);

    data.state.windows.space.map_output(&output, (0, 0));
    let mut damage_tracker = OutputDamageTracker::from_output(&output);
    let mut switcher_overlay = SwitcherOverlay::default();
    backend.window().request_redraw();

    event_loop
        .handle()
        .insert_source(winit, move |event, _, data| {
            let state = &mut data.state;
            let display_handle = &mut data.display_handle;

            match event {
                WinitEvent::Resized { size, .. } => {
                    output.change_current_state(
                        Some(Mode {
                            size,
                            refresh: OUTPUT_REFRESH_MHZ,
                        }),
                        None,
                        None,
                        None,
                    );
                }
                WinitEvent::Input(event) => {
                    state.process_input_event(event);
                    backend.window().request_redraw();
                }
                WinitEvent::Redraw => {
                    crate::infra::frame_pacing::begin_frame(state);
                    let size = backend.window_size();

                    let scale = Scale::from(1.0);
                    let oriented_size = output.current_transform().transform_size(size);

                    cursor.set_icon(state.effective_cursor_icon());
                    let cursor_element = if state.input.cursor_visible {
                        cursor.element(backend.renderer(), state.input.pointer_location, scale)
                    } else {
                        None
                    };

                    let below_windows = state.overlays.launcher.render(
                        backend.renderer(),
                        (oriented_size.w, oriented_size.h),
                        scale,
                    );
                    let above_windows = switcher_overlay.render(
                        backend.renderer(),
                        state.overlays.alt_tab_switcher.as_ref(),
                        (oriented_size.w, oriented_size.h),
                        scale,
                    );

                    let render_elements: Vec<CustomRenderElements> = {
                        let ctx = FrameContext {
                            output: &output,
                            scale,
                            cursor: cursor_element,
                            below_windows,
                            above_windows,
                            space: &state.windows.space,
                            blur_engine: Some(&state.resources.render.blur_engine),
                            pointer: state.input.pointer_location,
                            visuals: &crate::modules::window::ops::window_animation_frame,
                        };
                        compose_frame(backend.renderer(), ctx)
                    };

                    let render_damage = {
                        let age = backend.buffer_age().unwrap_or(0);
                        let (renderer, mut framebuffer) =
                            backend.bind().expect("failed to bind Winit framebuffer");
                        let render_result = damage_tracker
                            .render_output(
                                renderer,
                                &mut framebuffer,
                                age,
                                &render_elements,
                                theme.background_color,
                            )
                            .expect("failed to render desktop frame");
                        render_result.damage.cloned()
                    };

                    if crate::modules::shell::screenshot::capture::take_requested() {
                        // Offscreen capture binds an FBO, which leaves the EGL
                        // context current *without* the window surface. Restore
                        // it or the following swap_buffers() fails with BadAlloc
                        // and takes down the compositor.
                        let surface = backend.egl_surface() as *const _;
                        let _ = crate::modules::shell::screenshot::capture::capture_screen(
                            backend.renderer(),
                            &render_elements,
                            &output,
                        );
                        let renderer = backend.renderer();
                        // SAFETY: Backend surface and renderer stay alive for this call.
                        unsafe {
                            let _ = renderer.egl_context().make_current_with_surface(&*surface);
                        }
                    }

                    backend
                        .submit(render_damage.as_deref())
                        .expect("failed to submit Winit frame");
                    crate::infra::frame_pacing::send_frames(state, &output);
                    crate::infra::frame_pacing::end_frame(state, display_handle);
                    backend.window().request_redraw();
                }
                WinitEvent::CloseRequested => state.resources.loop_signal.stop(),
                _ => {}
            }
        })?;

    Ok(())
}
