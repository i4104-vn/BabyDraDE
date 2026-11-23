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
    application::autostart,
    infrastructure::{backends, process::ProcessLauncher},
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
pub(crate) fn run_loop(mut wiring: Wiring<'_>) -> Result<(), Box<dyn Error>> {
    let Wiring {
        event_loop,
        data,
        host_has_display,
    } = &mut wiring;

    let (clipboard_tx, clipboard_rx) =
        smithay::reexports::calloop::channel::channel::<Option<String>>();
    data.state.clipboard_sender = Some(clipboard_tx);
    event_loop
        .handle()
        .insert_source(clipboard_rx, |event, _, data| {
            if let Event::Msg(mime) = event {
                data.state.complete_clipboard_capture(mime);
            }
        })
        .map_err(|err| ClipboardChannelError(err.error))?;

    backends::init(event_loop, data, *host_has_display)?;

    // Convertible devices can rotate the panel from their accelerometer.
    crate::infrastructure::sensor::auto_rotate::spawn(event_loop);

    // Only now publish our own socket; GTK and the backend already started.
    bootstrap::export_wayland_display(&data.state.socket_name);

    info!(
        "BabyDraDE is running. Wayland socket: {}",
        data.state.socket_name.to_string_lossy()
    );

    let autostart_children = autostart::start(
        &services::config::load_autostart_config().commands,
        &ProcessLauncher,
        data.state.socket_name.as_os_str(),
    );

    let run_res = event_loop.run(None, data, |_| {});

    info!("BabyDraDE compositor stopping, terminating autostart processes...");
    for mut child in autostart_children {
        let _ = child.kill();
        let _ = child.wait();
    }

    run_res?;
    Ok(())
}
