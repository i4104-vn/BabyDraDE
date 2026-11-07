mod backends;
mod compositor;

use std::{error::Error, ffi::OsString, process::Command};

use smithay::reexports::{
    calloop::EventLoop,
    wayland_server::Display,
};
use tracing::{error, info};

use compositor::{BabydraState, CalloopData};

pub fn run(test_app: Option<OsString>) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("XDG_RUNTIME_DIR").is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "XDG_RUNTIME_DIR is required; run BabyDraDE from a Linux desktop session",
        )
        .into());
    }

    let mut event_loop: EventLoop<CalloopData> = EventLoop::try_new()?;
    let display: Display<BabydraState> = Display::new()?;
    let display_handle = display.handle();
    let state = BabydraState::new(&mut event_loop, display);

    let mut data = CalloopData {
        state,
        display_handle,
    };

    backends::winit_backend::init(&mut event_loop, &mut data)?;

    let app = test_app
        .or_else(|| std::env::var_os("BABYDRADE_TEST_APP"))
        .unwrap_or_else(|| OsString::from("foot"));
    match Command::new(&app).spawn() {
        Ok(_child) => info!("Started test app: {}", app.to_string_lossy()),
        Err(err) => error!(
            "Could not start test app '{}': {}. Set BABYDRADE_TEST_APP or pass --app.",
            app.to_string_lossy(),
            err
        ),
    }

    info!(
        "BabyDraDE is running. Wayland socket: {}",
        data.state.socket_name.to_string_lossy()
    );
    event_loop.run(None, &mut data, |_| {})?;
    Ok(())
}
