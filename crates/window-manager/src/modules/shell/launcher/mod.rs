//! Compositor-owned application launcher: a docked vertical notch on the left
//! edge, styled after the topbar notch (near-black surface, hairline white border,
//! smooth concave fillets transition to the edge, rounded outer corners). Shown
//! via its shortcut and navigated with arrow keys, the wheel or the pointer.
//!
//! Geometry comes from `ui_kit::components::dock`; [`render`] owns Cairo drawing
//! and [`buffer`] owns off-screen pixel work. [`state`] holds selection and
//! input state; [`search`] is the expanded overlay that takes over as soon as
//! the user types while the dock is open.

pub(crate) use common::constants::launcher::{
    BACKDROP_WIDTH_MIN, BACKDROP_WIDTH_RATIO, LABEL_FONT_SIZE, LABEL_GAP, LABEL_HEIGHT,
    LABEL_MAX_TEXT, LABEL_MAX_WIDTH, LABEL_PAD, SLOT,
};

pub(crate) use models::{BackdropKey, BufferRegion, CacheKey, LauncherApp, LauncherState};

pub(crate) mod backdrop;
pub(crate) mod models;
pub(crate) mod render;
mod render_tests;
pub(crate) mod search;
pub(crate) mod state;
