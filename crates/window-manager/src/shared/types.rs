//! Entities and pure rules — the innermost Clean Architecture layer.
//!
//! Nothing here reads config, starts processes, talks to a window-system
//! protocol or touches GTK. Geometry is represented by the framework-free
//! value types in [`geometry`].
//!
//! This file is a facade: the value models themselves live in
//! [`crate::shared::models`].

pub(crate) use crate::shared::models::geometry;
pub(crate) use crate::shared::models::keysyms;
pub(crate) use crate::shared::models::titlebar::{self, TitlebarLayout};
#[allow(unused_imports)]
pub(crate) use crate::shared::models::window::{self, WindowMetadata, WindowVisualState};
