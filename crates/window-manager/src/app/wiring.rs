//! Dependency construction for the composition root.
//!
//! This module only *builds* the compositor's objects. It deliberately does
//! not start the event loop, register sources or talk to backends — those are
//! lifecycle concerns and live in [`super::runtime`].

use std::error::Error;
use std::sync::Arc;

use smithay::reexports::{calloop::EventLoop, wayland_server::Display};

use crate::{
    app::state::{BabydraState, CalloopData},
    core::shortcuts::{ShortcutRegistry, ShortcutService},
    infra::process::process::ShortcutCommandLauncher,
    shared::config::{gesture_policy, keymap as config},
    shortcuts as api,
};

use super::bootstrap;

/// Everything the runtime needs to drive the compositor.
pub(crate) struct Wiring<'a> {
    pub(crate) event_loop: EventLoop<'a, CalloopData>,
    pub(crate) data: CalloopData,
    /// Whether a host session was present *before* we exported our own
    /// `WAYLAND_DISPLAY`; used by the backend to decide if it is nested.
    pub(crate) host_has_display: bool,
}

/// Builds the state by injecting concrete config/process adapters into the
/// shortcut use case, then constructing the event loop, display and state.
///
/// Order matters and is preserved from the original composition root:
/// 1. the shortcut service and its hot-reload watcher are started;
/// 2. the *host* display environment is detected — this must happen before
///    exporting our own `WAYLAND_DISPLAY`, or the backend would always think
///    it is nested;
/// 3. the event loop, Wayland display and compositor state are created;
/// 4. toolkit hints are exported and GTK is initialized while
///    `WAYLAND_DISPLAY` still points at the host session (if any);
/// 5. the per-user Babydra directories are ensured and `CalloopData` is
///    assembled.
///
/// The runtime publishes our own socket only after the backend has started.
///
/// # Returns
/// The event loop, loop data and host-display flag the runtime needs.
///
/// # Errors
/// Returns an error when the shortcut service, event loop, display, protocol
/// bus or compositor state cannot be constructed.
pub(crate) fn build_state<'a>() -> Result<Wiring<'a>, Box<dyn Error>> {
    // Composition root: inject concrete config and process adapters into the
    // shortcut use case, then start its hot-reload watcher.
    let shortcuts = Arc::new(ShortcutService::new(
        Arc::new(config::KeymapRepository),
        Arc::new(ShortcutCommandLauncher),
    ));
    config::spawn_watcher(shortcuts.shared().clone());

    // Runtime registrations live outside the reloadable keymap table; features
    // publish them through the protocol bus instead of a watched directory.
    let registry = Arc::new(ShortcutRegistry::new());
    api::publish(registry.clone());

    // Capture the *host* display before exporting our own WAYLAND_DISPLAY,
    // otherwise the backend would always think it is nested.
    let host_has_display = std::env::var_os("WAYLAND_DISPLAY").is_some()
        || std::env::var_os("WAYLAND_SOCKET").is_some()
        || std::env::var_os("DISPLAY").is_some();

    let mut event_loop: EventLoop<'a, CalloopData> = EventLoop::try_new()?;
    let display: Display<BabydraState> = Display::new()?;
    let display_handle = display.handle();

    // The bus socket must exist before any autostart feature tries to connect.
    let (bus_host, bus_sender) = super::bus::build()?;
    let state = BabydraState::new(
        &mut event_loop,
        display,
        gesture_policy,
        shortcuts.clone(),
        registry.clone(),
        bus_sender,
    )?;

    // Toolkit hints must be visible to GTK, but WAYLAND_DISPLAY must still
    // point at the host session here (if any) until the backend is chosen.
    bootstrap::export_toolkit_env();
    bootstrap::init_gtk();
    services::config::ensure_babydra_dirs();

    let mut data = CalloopData {
        state,
        display_handle,
        bus: bus_host,
    };
    super::bus::init(&mut event_loop, &mut data)?;

    Ok(Wiring {
        event_loop,
        data,
        host_has_display,
    })
}
