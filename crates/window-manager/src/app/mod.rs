//! Application composition root: lifecycle, bootstrap and subcommand dispatch.

use std::error::Error;

pub(crate) mod bootstrap;
pub(crate) mod bridge;
pub(crate) mod bus;
pub(crate) mod models;
pub(crate) mod runtime;
pub(crate) mod state;
pub(crate) mod wiring;

// Compatibility re-exports: keep `crate::app::autostart`, `crate::app::commands`
// and `crate::app::input_method` resolvable for existing callers.
pub(crate) use bootstrap::commands;

/// Boots the compositor and starts the event loop.
pub(crate) fn run() -> Result<(), Box<dyn Error>> {
    bootstrap::init_tracing();
    if commands::dispatch() {
        return Ok(());
    }
    bootstrap::ensure_runtime_dir();
    let state = wiring::build_state()?;
    runtime::run_loop(state)
}
