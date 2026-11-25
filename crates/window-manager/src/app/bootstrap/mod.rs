//! Environment and toolkit bootstrap for the compositor process.

#[allow(clippy::module_inception)]
pub(crate) mod autostart;
pub(crate) mod commands;
pub(crate) mod init;
pub(crate) mod input_method;

pub(crate) use init::{
    ensure_runtime_dir, export_toolkit_env, export_wayland_display, init_gtk, init_tracing,
};
