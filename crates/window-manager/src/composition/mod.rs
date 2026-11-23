//! Application composition root: lifecycle, bootstrap and subcommand dispatch.

pub(crate) mod bootstrap;
pub(crate) mod commands;
pub(crate) mod input_method;
pub(crate) mod runtime;
pub(crate) mod wiring;

use std::error::Error;

pub(crate) fn run() -> Result<(), Box<dyn Error>> {
    // Bootstrap first: the runtime dir must exist before any socket is made,
    // and wiring must detect a host display before we export our own socket.
    bootstrap::ensure_runtime_dir();
    let state = wiring::build_state()?;
    runtime::run_loop(state)
}
