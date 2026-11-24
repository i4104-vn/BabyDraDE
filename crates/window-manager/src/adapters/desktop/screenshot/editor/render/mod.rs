//! Editor window UI construction: overlay layout, glassmorphic toolbar, and canvas.

mod color;
mod models;
mod render;
mod toolbar;
mod tools;

pub(crate) use render::build_editor_ui;
