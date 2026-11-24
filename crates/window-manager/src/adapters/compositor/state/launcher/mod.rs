//! Launcher state facade.
//! Input handling lives in [`input`]; buffer assembly lives in [`render`].

mod context;
mod input;
mod models;
mod render;
mod search;
mod surface;

pub(crate) use models::LauncherState;
