//! Plain data types shared across compositor layers.
//!
//! This module is a leaf: it depends only on external libraries and never on
//! other layers of `crates/window-manager`.

pub(crate) mod titlebar;
pub(crate) mod window;

pub(crate) use titlebar::TitlebarLayout;
pub(crate) use window::WindowVisualState;
