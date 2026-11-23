//! Entities and pure rules — the innermost Clean Architecture layer.
//!
//! Nothing here reads config, starts processes, talks to a window-system
//! protocol or touches GTK. `smithay::utils` geometry primitives are used as
//! value objects only.

pub(crate) mod keysyms;
pub(crate) mod shortcuts;
pub(crate) mod titlebar;
pub(crate) mod window;

pub(crate) use titlebar::TitlebarLayout;
pub(crate) use window::WindowVisualState;
