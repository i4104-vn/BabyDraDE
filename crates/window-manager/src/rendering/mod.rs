pub mod animation;
pub mod blur;
pub mod cursor;
pub mod elements;
pub(crate) mod lock;
pub mod switcher;
pub mod titlebar;

pub use animation::{render_animated_window, WindowAnimationRenderParams};
pub use blur::{BlurEngine, BlurParams, BlurRenderElement};
pub use cursor::Cursor;
pub use elements::{render_layer_surfaces, render_window, CustomRenderElements};
pub use switcher::SwitcherOverlay;
