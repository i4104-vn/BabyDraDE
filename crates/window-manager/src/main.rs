// TODO(phase5): wrap unsafe fn bodies in explicit unsafe blocks, then remove.
#![allow(unsafe_op_in_unsafe_fn)]

mod app;
mod backends;
mod compositor;
mod input;
mod keymap;
mod model;
mod platform;
pub(crate) mod rendering;
mod shell;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    app::bootstrap::init_tracing();
    if app::commands::dispatch() {
        return Ok(());
    }
    app::run()
}
