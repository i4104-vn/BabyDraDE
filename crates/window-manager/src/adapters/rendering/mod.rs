//! Core rendering: system/window elements, window animations, decorations and blur.

pub(crate) mod animation;
pub(crate) mod blur;
pub(crate) mod buffer;
pub(crate) mod elements;
pub(crate) mod frame;
pub(crate) mod titlebar;

pub(crate) use elements::CustomRenderElements;
pub(crate) use frame::{compose_frame, FrameContext};
