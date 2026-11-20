//! Core rendering: system/window elements, window animations, decorations and blur.

pub mod animation;
pub mod blur;
pub mod elements;
pub mod titlebar;

pub use animation::{render_animated_window, WindowAnimationRenderParams};
pub use blur::{BlurEngine, BlurParams, BlurRenderElement};
pub use elements::{render_layer_surfaces, render_window, CustomRenderElements};
