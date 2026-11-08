use std::{error::Error, time::Duration};

use presentation::DesktopTheme;
use smithay::{
    backend::{
        renderer::{damage::OutputDamageTracker, glow::GlowRenderer},
        winit::{self, WinitEvent},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::calloop::EventLoop,
    utils::{Rectangle, Scale, Transform},
};

use crate::{
    compositor::{BabydraState, CalloopData},
    rendering::{render_window_with_borders, BorderShader, CornerShader, Cursor, CustomRenderElements},
};

pub(crate) fn init(
    event_loop: &mut EventLoop<CalloopData>,
    data: &mut CalloopData,
) -> Result<(), Box<dyn Error>> {
    let (mut backend, winit) = winit::init::<GlowRenderer>().map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "Failed to initialize graphical window (Winit backend): {}\n\
                 Ensure a Wayland host is running, or if running directly on TTY, run via:\n\
                   WLR_RENDERER=pixman cage -- cargo run\n\
                 or run ./start.sh",
                err
            ),
        )
    })?;
    backend.window().set_title("BabyDraDE");
    backend.window().set_cursor_visible(true);

    // Initialize custom GLSL shaders (borders, rounded corners) inspired by MagmaWM
    BorderShader::init(backend.renderer());
    CornerShader::init(backend.renderer());

    let cursor = Cursor::new();
    let theme = DesktopTheme::magma_cyan();

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
    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        None,
        Some((0, 0).into()),
    );
    output.set_preferred(mode);

    data.state.space.map_output(&output, (0, 0));
    std::env::set_var("WAYLAND_DISPLAY", &data.state.socket_name);

    let mut damage_tracker = OutputDamageTracker::from_output(&output);
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
                    let damage = Rectangle::from_size(size);

                    let mut render_elements: Vec<CustomRenderElements> = Vec::new();
                    let scale = Scale::from(1.0);

                    // 1. Mouse cursor MUST BE FIRST in front-to-back order (index 0 is on top of everything)
                    if let Some(cursor_elem) = cursor.element(backend.renderer(), state.pointer_location, scale) {
                        render_elements.push(cursor_elem);
                    }

                    // 2. Render each mapped window in front-to-back order (highest z-index / focused window first)
                    for window in state.space.elements().rev() {
                        if let Some(loc) = state.space.element_location(window) {
                            render_elements.extend(render_window_with_borders(
                                backend.renderer(),
                                window,
                                loc,
                                scale,
                                &theme.border,
                            ));
                        }
                    }

                    {
                        let (renderer, mut framebuffer) =
                            backend.bind().expect("failed to bind Winit framebuffer");
                        damage_tracker
                            .render_output(
                                renderer,
                                &mut framebuffer,
                                0,
                                &render_elements,
                                theme.background_color,
                            )
                            .expect("failed to render desktop frame");
                    }

                    BorderShader::cleanup(backend.renderer());

                    backend
                        .submit(Some(&[damage]))
                        .expect("failed to submit Winit frame");
                    state.space.elements().for_each(|window| {
                        window.send_frame(
                            &output,
                            state.start_time.elapsed(),
                            Some(Duration::ZERO),
                            |_, _| Some(output.clone()),
                        );
                    });

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
