// calloop's level event source requires an unsafe callback signature.
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::items_after_test_module, clippy::module_inception)]
//! BabyDraDE compositor library.
//!
//! The binary entry point calls [`run`]. Other in-process components can use
//! [`shortcuts`] to register runtime shortcuts without reaching into private
//! internals; out-of-process apps use `services::system::shortcuts` instead.

/// Public in-process shortcut registration API.
pub mod shortcuts;

mod app;
mod core;
mod infra;
mod modules;
mod shared;

use std::error::Error;

/// Boots and runs the compositor until its event loop stops.
pub fn run() -> Result<(), Box<dyn Error>> {
    app::run()
}
