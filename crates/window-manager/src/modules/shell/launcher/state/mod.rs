//! Launcher state facade.
//! Input handling lives in [`crate::modules::shell::launcher::models`]; buffer assembly lives in [`render`].

pub(crate) use crate::modules::shell::launcher::models::LauncherState;

mod context;
mod render;
mod surface;
