//! DRM/GPU discovery and initialization.

mod device;
mod handler;
mod models;
mod scan;
mod surfaces;

pub(crate) use scan::scan_and_init_gpu;
