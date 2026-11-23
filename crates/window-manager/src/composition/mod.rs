//! Application composition root: lifecycle, bootstrap and subcommand dispatch.

pub(crate) mod bootstrap;
pub(crate) mod commands;
pub(crate) mod input_method;

use std::error::Error;

use smithay::reexports::{calloop::EventLoop, wayland_server::Display};
use tracing::{info, warn};

use crate::{
    adapters::{
        compositor::{BabydraState, CalloopData},
        input::keymap::config,
    },
    application::shortcuts::ShortcutRegistry,
    infrastructure::{backends, config::gesture_policy, process::spawn_detached},
};

pub(crate) fn run() -> Result<(), Box<dyn Error>> {
    bootstrap::ensure_runtime_dir();

    // Composition root: inject concrete config and process adapters into the
    // shortcut use case, then start its hot-reload watcher.
    let shortcuts = ShortcutRegistry::init(config::load, spawn_detached);
    config::spawn_watcher(shortcuts.shared().clone());

    // Capture the *host* display before exporting our own WAYLAND_DISPLAY,
    // otherwise the backend would always think it is nested.
    let host_has_display = std::env::var_os("WAYLAND_DISPLAY").is_some()
        || std::env::var_os("WAYLAND_SOCKET").is_some()
        || std::env::var_os("DISPLAY").is_some();

    let mut event_loop: EventLoop<'_, CalloopData> = EventLoop::try_new()?;
    let display: Display<BabydraState> = Display::new()?;
    let display_handle = display.handle();
    let state = BabydraState::new(&mut event_loop, display, gesture_policy);

    // Toolkit hints must be visible to GTK, but WAYLAND_DISPLAY must still
    // point at the host session here (if any) until the backend is chosen.
    bootstrap::export_toolkit_env();
    bootstrap::init_gtk();
    services::config::ensure_babydra_dirs();

    let mut data = CalloopData {
        state,
        display_handle,
    };

    let (clipboard_tx, clipboard_rx) =
        smithay::reexports::calloop::channel::channel::<Option<String>>();
    data.state.clipboard_sender = Some(clipboard_tx);
    event_loop
        .handle()
        .insert_source(clipboard_rx, |event, _, data| {
            if let smithay::reexports::calloop::channel::Event::Msg(mime) = event {
                data.state.complete_clipboard_capture(mime);
            }
        })
        .expect("failed to register clipboard completion channel");

    backends::init(&mut event_loop, &mut data, host_has_display)?;

    // Only now publish our own socket; GTK and the backend already started.
    bootstrap::export_wayland_display(&data.state.socket_name);

    info!(
        "BabyDraDE is running. Wayland socket: {}",
        data.state.socket_name.to_string_lossy()
    );

    let mut autostart_children = Vec::new();
    for cmd_str in &services::config::load_autostart_config().commands {
        let trimmed = cmd_str.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        info!("Spawning autostart command: {trimmed}");
        match std::process::Command::new("sh")
            .arg("-c")
            .arg(trimmed)
            .env("WAYLAND_DISPLAY", &data.state.socket_name)
            .spawn()
        {
            Ok(child) => autostart_children.push(child),
            Err(err) => warn!("Failed to spawn autostart command '{trimmed}': {err}"),
        }
    }

    let run_res = event_loop.run(None, &mut data, |_| {});

    info!("BabyDraDE compositor stopping, terminating autostart processes...");
    for mut child in autostart_children {
        let _ = child.kill();
        let _ = child.wait();
    }

    run_res?;
    Ok(())
}
