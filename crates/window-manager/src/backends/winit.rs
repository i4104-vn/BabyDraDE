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
    compositor::{BabydraState, CalloopData},
    rendering::{
        render_layer_surfaces, render_window, Cursor, CustomRenderElements, SwitcherOverlay,
    },
};

pub(crate) fn init(
    event_loop: &mut EventLoop<CalloopData>,
    data: &mut CalloopData,
) -> Result<(), Box<dyn Error>> {
    let (mut backend, winit) = winit::init::<GlowRenderer>().map_err(|err| {
        std::io::Error::other(format!(
            "Failed to initialize graphical window (Winit backend): {}\n\
             Ensure a Wayland host is running, or if running directly on TTY, run via:\n\
               WLR_RENDERER=pixman cage -- cargo run\n\
             or run ./start.sh",
            err
        ))
    })?;
    backend.window().set_title("BabyDraDE");
    backend.window().set_cursor_visible(true);

    let cursor = Cursor::new();
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
        refresh: 60_000,
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
                            refresh: 60_000,
                        }),
                        None,
                        None,
                        None,
                    );
                }
                WinitEvent::Input(event) => state.process_input_event(event),
                WinitEvent::Redraw => {
                    let size = backend.window_size();

                    let mut render_elements: Vec<CustomRenderElements> = Vec::new();
                    let scale = Scale::from(1.0);
                    let blur_engine = Some(&state.blur_engine);

                    // 1. Mouse cursor MUST BE FIRST in front-to-back order (index 0 is on top of everything)
                    if let Some(cursor_elem) =
                        cursor.element(backend.renderer(), state.pointer_location, scale)
                    {
                        render_elements.push(cursor_elem);
                    }

                    let oriented_size = output.current_transform().transform_size(size);
                    if let Some(element) = switcher_overlay.element(
                        backend.renderer(),
                        state.alt_tab_switcher.as_ref(),
                        (oriented_size.w, oriented_size.h),
                    ) {
                        render_elements.push(element);
                    }

                    // 2. Render Overlay & Top layer surfaces (topbar, dynamic island, notification popups)
                    render_elements.extend(render_layer_surfaces(
                        backend.renderer(),
                        &output,
                        scale,
                        &[
                            smithay::wayland::shell::wlr_layer::Layer::Overlay,
                            smithay::wayland::shell::wlr_layer::Layer::Top,
                        ],
                        blur_engine,
                    ));

                    // 3. Render each mapped window in front-to-back order (highest z-index / focused window first)
                    for window in state.space.elements().rev() {
                        if let Some(loc) = state.space.element_location(window) {
                            render_elements.extend(render_window(
                                backend.renderer(),
                                window,
                                loc,
                                scale,
                                blur_engine,
                            ));
                        }
                    }

                    // 4. Render Bottom & Background layer surfaces (wallpaper, desktop background)
                    render_elements.extend(render_layer_surfaces(
                        backend.renderer(),
                        &output,
                        scale,
                        &[
                            smithay::wayland::shell::wlr_layer::Layer::Bottom,
                            smithay::wayland::shell::wlr_layer::Layer::Background,
                        ],
                        blur_engine,
                    ));

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
                    backend.window().set_cursor_visible(true);
                    backend.window().request_redraw();
                }
                WinitEvent::CloseRequested => state.loop_signal.stop(),
                _ => {}
            }
        })?;

    Ok(())
}
