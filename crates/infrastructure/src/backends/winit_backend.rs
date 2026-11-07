use std::{error::Error, time::Duration};

use smithay::{
    backend::{
        renderer::{
            damage::OutputDamageTracker, element::surface::WaylandSurfaceRenderElement,
            gles::GlesRenderer,
        },
        winit::{self, WinitEvent},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{
        calloop::EventLoop,
        wayland_server::DisplayHandle,
    },
    utils::{Rectangle, Transform},
};

use crate::compositor::{BabydraState, CalloopData};

pub(crate) fn init(
    event_loop: &mut EventLoop<CalloopData>,
    data: &mut CalloopData,
) -> Result<(), Box<dyn Error>> {
    let (mut backend, winit) = winit::init::<GlesRenderer>()?;
    backend.window().set_title("BabyDraDE");
    backend.window().set_cursor_visible(true);

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
    output.change_current_state(Some(mode), Some(Transform::Flipped180), None, Some((0, 0).into()));
    output.set_preferred(mode);

    data.state.space.map_output(&output, (0, 0));
    std::env::set_var("WAYLAND_DISPLAY", &data.state.socket_name);

    let mut damage_tracker = OutputDamageTracker::from_output(&output);
    backend.window().request_redraw();

    event_loop
        .handle()
        .insert_source(winit, move |event, _, data| {
            let state = &mut data.state;
            let display_handle: &DisplayHandle = &data.display_handle;

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

                    {
                        let (renderer, mut framebuffer) =
                            backend.bind().expect("failed to bind Winit framebuffer");
                        smithay::desktop::space::render_output::<
                            _,
                            WaylandSurfaceRenderElement<GlesRenderer>,
                            _,
                            _,
                        >(
                            &output,
                            renderer,
                            &mut framebuffer,
                            1.0,
                            0,
                            [&state.space],
                            &[],
                            &mut damage_tracker,
                            [0.0, 0.0, 0.0, 1.0],
                        )
                        .expect("failed to render desktop frame");
                    }

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
