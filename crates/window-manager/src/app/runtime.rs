//! Event-loop lifecycle: source registration, backend startup, autostart and
//! shutdown.
//!
//! Dependency construction lives in [`super::wiring`]; this module only
//! drives the objects it is handed.

use std::error::Error;
use std::fmt;

use smithay::reexports::calloop::{channel::Event, Error as CalloopError};
use tracing::info;

use crate::{
    app::bootstrap::autostart,
    infra::{self as backends, process::process::ProcessLauncher},
};

use super::{bootstrap, wiring::Wiring};

/// Adds context to a failure to register the clipboard completion channel.
#[derive(Debug)]
struct ClipboardChannelError(CalloopError);

impl fmt::Display for ClipboardChannelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("failed to register clipboard completion channel")
    }
}

impl Error for ClipboardChannelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

/// Runs the compositor until its event loop stops, cleaning up autostart
/// processes afterwards.
///
/// # Arguments
/// * `wiring` - Built dependencies the loop drives.
///
/// # Errors
/// Returns an error when the clipboard channel, a backend or the event loop
/// itself fails.
pub(crate) fn run_loop(mut wiring: Wiring<'_>) -> Result<(), Box<dyn Error>> {
    let Wiring {
        event_loop,
        data,
        host_has_display,
    } = &mut wiring;

    let (clipboard_tx, clipboard_rx) =
        smithay::reexports::calloop::channel::channel::<Option<String>>();
    data.state.clipboard.sender = Some(clipboard_tx);
    event_loop
        .handle()
        .insert_source(clipboard_rx, |event, _, data| {
            if let Event::Msg(mime) = event {
                data.state.complete_clipboard_capture(mime);
            }
        })
        .map_err(|err| ClipboardChannelError(err.error))?;

    backends::init(event_loop, data, *host_has_display)?;

    // Features may be waiting for the compositor before starting their UI.
    let _ = data
        .state
        .bus
        .publish("bus/ready", &crate::protocol::Event::CompositorReady);

    // Convertible devices can rotate the panel from their accelerometer.
    crate::infra::process::auto_rotate::spawn(event_loop);

    // Only now publish our own socket; GTK and the backend already started.
    bootstrap::export_wayland_display(&data.state.resources.socket_name);

    info!(
        "BabyDraDE is running. Wayland socket: {}",
        data.state.resources.socket_name.to_string_lossy()
    );

    let children = autostart::start(
        &services::config::load_autostart_config().commands,
        &ProcessLauncher,
        data.state.resources.socket_name.as_os_str(),
    );

    let run_res = event_loop.run(None, data, |_| {});

    info!("BabyDraDE compositor stopping, terminating autostart processes...");
    for mut child in children {
        let _ = child.kill();
        let _ = child.wait();
    }

    let _ = data
        .state
        .bus
        .publish("bus/shutdown", &crate::protocol::Event::Shutdown);
    data.state.bus.shutdown();

    run_res?;
    Ok(())
}
