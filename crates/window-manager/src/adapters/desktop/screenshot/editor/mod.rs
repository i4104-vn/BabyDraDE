//! Screenshot editor window assembly.

pub(crate) mod canvas;
pub(crate) mod handle;
pub(crate) mod models;
pub(crate) mod popovers;

mod handler;
mod render;

pub(crate) use handler::setup_editor_keys;
pub(crate) use render::build_editor_ui;
