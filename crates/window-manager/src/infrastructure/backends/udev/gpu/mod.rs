//! DRM/GPU discovery and initialization.

mod handler;
mod models;

pub(crate) use handler::scan_and_init_gpu;
