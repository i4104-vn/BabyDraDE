// calloop's level event source requires an unsafe callback signature.
#![allow(unsafe_op_in_unsafe_fn)]
mod adapters;
mod application;
mod composition;
mod domain;
mod infrastructure;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    composition::bootstrap::init_tracing();
    if composition::commands::dispatch() {
        return Ok(());
    }
    composition::run()
}
