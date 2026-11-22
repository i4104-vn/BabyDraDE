use std::{error::Error, time::Duration};

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
    compositor::{BabydraState, CalloopData, Cursor},
    rendering::{compose_frame, CustomRenderElements, FrameContext},
    shell::SwitcherOverlay,
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
    let transform = match std::env::var("BABYDRADE_TRANSFORM").as_deref() {
        Ok("90") => Transform::Flipped90,
        Ok("180") => Transform::Normal,
        Ok("270") => Transform::Flipped270,
        _ => Transform::Flipped180,
    };
    output.change_current_state(Some(mode), Some(transform), None, Some((0, 0).into()));
    output.set_preferred(mode);

    data.state.space.map_output(&output, (0, 0));
    std::env::set_var("WAYLAND_DISPLAY", &data.state.socket_name);

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
                    if let Some(action) = services::system::window::read_and_clear_window_action() {
                        state.handle_window_action(&action);
                    }
                    state.update_window_animations();
                    state.update_clipboard();
                    let size = backend.window_size();

                    let scale = Scale::from(1.0);
                    let oriented_size = output.current_transform().transform_size(size);

                    cursor.set_icon(state.effective_cursor_icon());
                    let cursor_element = if state.cursor_visible {
                        cursor.element(backend.renderer(), state.pointer_location, scale)
                    } else {
                        None
                    };

                    let below_windows = state.launcher.render(
                        backend.renderer(),
                        (oriented_size.w, oriented_size.h),
                        scale,
                    );
                    let above_windows = switcher_overlay.render(
                        backend.renderer(),
                        state.alt_tab_switcher.as_ref(),
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
                            space: &state.space,
                            blur_engine: Some(&state.blur_engine),
                            pointer: state.pointer_location,
                            visuals: &crate::compositor::window_animation_frame,
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

                    if crate::shell::screenshot::capture::take_requested() {
                        // Offscreen capture binds an FBO, which leaves the EGL
                        // context current *without* the window surface. Restore
                        // it or the following swap_buffers() fails with BadAlloc
                        // and takes down the compositor.
                        let surface = backend.egl_surface() as *const _;
                        let _ = crate::shell::screenshot::capture::capture_screen(
                            backend.renderer(),
                            &render_elements,
                            &output,
                        );
                        let renderer = backend.renderer();
                        unsafe {
                            let _ = renderer.egl_context().make_current_with_surface(&*surface);
                        }
                    }

                    backend
                        .submit(render_damage.as_deref())
                        .expect("failed to submit Winit frame");
                    state.space.elements().for_each(|window| {
                        window.send_frame(
                            &output,
                            state.start_time.elapsed(),
                            Some(Duration::ZERO),
                            |_, _| Some(output.clone()),
                        );
                    });

                    let layer_map = smithay::desktop::layer_map_for_output(&output);
                    for layer in layer_map.layers() {
                        layer.send_frame(
                            &output,
                            state.start_time.elapsed(),
                            Some(Duration::ZERO),
                            |_, _| Some(output.clone()),
                        );
                    }

                    state.space.refresh();
                    state.popups.cleanup();
                    let _ = display_handle.flush_clients();
                    backend.window().request_redraw();
                }
                WinitEvent::CloseRequested => state.loop_signal.stop(),
                _ => {}
            }
        })?;

    Ok(())
}
