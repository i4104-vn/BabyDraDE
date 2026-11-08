//! BabyDraDE UI Kit — Reusable GTK4 Components and Fluent Stylesheets
//!
//! Provides modular components (`PillBox`, `VerticalSeparator`, `Badge`, `CircularDisc`, `FluentButton`, `Icon`)
//! and Fluent styling for desktop widgets.

pub mod components;
pub mod style;

pub use components::*;
pub use style::{load_ui_kit_styles, FLUENT_DARK, FLUENT_LIGHT};

