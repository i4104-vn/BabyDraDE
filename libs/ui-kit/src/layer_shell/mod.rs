//! Reusable GTK4 layer-shell window setup.

pub mod context;
pub mod handler;

pub use context::LayerWindowConfig;
pub use handler::init_layer_window;
