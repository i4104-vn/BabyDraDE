//! Plain data types shared across compositor layers.
//!
//! This module is a leaf: it depends only on external libraries and never on
//! other layers of `crates/window-manager`.

pub mod titlebar;

pub use titlebar::TitlebarLayout;
