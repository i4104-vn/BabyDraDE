// calloop's level event source requires an unsafe callback signature.
#![allow(unsafe_op_in_unsafe_fn)]
//! BabyDraDE compositor library.
//!
//! The binary entry point calls [`run`]. Other in-process components can use
//! [`shortcuts`] to register runtime shortcuts without reaching into private
//! layers; out-of-process apps use `services::system::shortcuts` instead.

/// Public in-process shortcut registration API.
pub mod shortcuts;

mod adapters;
mod application;
mod composition;
mod domain;
mod infrastructure;

use std::error::Error;

/// Boots and runs the compositor until its event loop stops.
///
/// # Returns
///
/// `Ok(())` on a clean shutdown, or the error that stopped the loop.
pub fn run() -> Result<(), Box<dyn Error>> {
    composition::bootstrap::init_tracing();
    if composition::commands::dispatch() {
        return Ok(());
    }
    composition::run()
}
