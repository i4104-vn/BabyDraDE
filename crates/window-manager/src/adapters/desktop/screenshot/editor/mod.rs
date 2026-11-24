//! Screenshot editor window assembly.

pub(crate) mod canvas;
pub(crate) mod handle;
pub(crate) mod model;
pub(crate) mod popovers;

mod handler;
mod view;

pub(crate) use handler::setup_editor_keys;
pub(crate) use view::build_editor_ui;
